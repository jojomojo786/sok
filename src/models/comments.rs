//! User-generated video comments: raw capture, KEmoji markup, and ammonia-sanitized HTML.
//!
//! ## Storage policy
//!
//! - **`body_raw`**: canonical user text after trim, newline collapse, and KEmoji normalization
//!   (`$#emoji_name#$` tokens). Inline HTML from non-KEmoji clients is stripped to text via
//!   `ammonia::clean_text` before storage.
//! - **`body_html`**: derived only through [`sanitize_comment_html`]. Templates and JSON must
//!   render **`body_html` only** (never `body_raw` or client-supplied HTML).
//!
//! Anonymous posting is allowed; blank display names are stored as `Guest` when handled by comment AJAX.

use ammonia::Builder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::sync::LazyLock;

/// Maximum stored length for comment text (`comments.body_raw`), in Unicode scalars.
pub const MAX_COMMENT_BODY_LEN: usize = 4_096;

/// Maximum stored length for display names (`comments.author_name`).
pub const MAX_AUTHOR_NAME_LEN: usize = 128;

/// Placeholder image used by legacy KEmoji `<img>` markup (see `static/js/main.min.js`).
pub const KEMOJI_PLACEHOLDER_IMG: &str = "/static/fox-tpl/style/img/opacity.png";

static KEMOJI_TEXT_TOKEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$#([^#$]+)#\$").expect("kemoji text token regex"));

static KEMOJI_HTML_IMG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<img[^>]*class="[^"]*\bke\s+ke-([^"\s]+)[^"]*"[^>]*>"#)
        .expect("kemoji html img regex")
});

static KEMOJI_HTML_ICON: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<i[^>]*class="[^"]*\bke\s+ke-([^"\s]+)[^"]*"[^>]*>\s*</i>"#)
        .expect("kemoji html icon regex")
});

static KEMOJI_BRACKET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]+)\]").expect("kemoji bracket regex"));

static INLINE_HTML_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<[^>]*>").expect("inline html tag regex"));

/// Validation or normalization failure when accepting a comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommentValidationError {
    AuthorNameEmpty,
    AuthorNameTooLong { max: usize },
    BodyEmpty,
    BodyTooLong { max: usize },
}

impl std::fmt::Display for CommentValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AuthorNameEmpty => write!(f, "author name is required"),
            Self::AuthorNameTooLong { max } => {
                write!(f, "author name must be at most {} characters", max)
            }
            Self::BodyEmpty => write!(f, "comment body is required"),
            Self::BodyTooLong { max } => {
                write!(f, "comment must be at most {} characters", max)
            }
        }
    }
}

impl std::error::Error for CommentValidationError {}

/// Values ready to persist on `comments` (`body_raw` + `body_html`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedCommentBody {
    pub body_raw: String,
    pub body_html: String,
}

/// Row-shaped comment for templates and APIs (maps to `comments` table).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comment {
    pub id: u64,
    pub video_id: u64,
    pub parent_id: Option<u64>,
    pub author_name: String,
    pub body_raw: String,
    pub body_html: String,
    pub is_visible: bool,
}

impl Comment {
    /// HTML safe for embedding in templates (already sanitized at write time).
    pub fn display_html(&self) -> &str {
        &self.body_html
    }
}

/// Normalize author display name for storage.
pub fn normalize_author_name(raw: &str) -> Result<String, CommentValidationError> {
    let trimmed: String = raw
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .trim()
        .to_string();

    if trimmed.is_empty() {
        return Err(CommentValidationError::AuthorNameEmpty);
    }
    if trimmed.chars().count() > MAX_AUTHOR_NAME_LEN {
        return Err(CommentValidationError::AuthorNameTooLong {
            max: MAX_AUTHOR_NAME_LEN,
        });
    }
    Ok(trimmed)
}

/// Normalize comment text for `body_raw` (no display HTML here).
pub fn normalize_body_raw(raw: &str) -> Result<String, CommentValidationError> {
    let mut text = raw.trim().to_string();
    if text.is_empty() {
        return Err(CommentValidationError::BodyEmpty);
    }

    if looks_like_kemoji_editor_html(&text) {
        text = kemoji_html_to_text(&text);
    } else if text.contains('<') {
        text = strip_inline_html_tags(&text);
    }

    text = collapse_raw_newlines(&text);
    if text.is_empty() {
        return Err(CommentValidationError::BodyEmpty);
    }
    if text.chars().count() > MAX_COMMENT_BODY_LEN {
        return Err(CommentValidationError::BodyTooLong {
            max: MAX_COMMENT_BODY_LEN,
        });
    }
    Ok(text)
}

/// Build `body_raw` and ammonia-sanitized `body_html` for insert.
pub fn prepare_comment_body(
    raw_input: &str,
) -> Result<PreparedCommentBody, CommentValidationError> {
    let body_raw = normalize_body_raw(raw_input)?;
    let body_html = sanitize_comment_html(&body_raw);
    Ok(PreparedCommentBody {
        body_raw,
        body_html,
    })
}

/// Convert KEmoji editor HTML (legacy `KEmoji.getValue(HTML_VALUE)`) to text tokens.
pub fn kemoji_html_to_text(html: &str) -> String {
    let mut out = html.to_string();
    out = KEMOJI_HTML_IMG
        .replace_all(&out, |caps: &regex::Captures| format!(" $#{}#$ ", &caps[1]))
        .into_owned();
    out = KEMOJI_HTML_ICON
        .replace_all(&out, |caps: &regex::Captures| format!(" $#{}#$ ", &caps[1]))
        .into_owned();
    out = out
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n");
    out = ammonia::clean_text(&out);
    collapse_raw_newlines(&out)
}

/// Expand `$#name#$` and `[name]` tokens into KEmoji `<i>` markup before ammonia runs.
pub fn expand_kemoji_tokens(text: &str) -> String {
    let mut out = KEMOJI_TEXT_TOKEN
        .replace_all(text, |caps: &regex::Captures| kemoji_icon_html(&caps[1]))
        .into_owned();
    out = KEMOJI_BRACKET
        .replace_all(&out, |caps: &regex::Captures| {
            let name = caps[1].trim();
            if is_plausible_emoji_name(name) {
                kemoji_icon_html(name)
            } else {
                caps[0].to_string()
            }
        })
        .into_owned();
    out
}

/// Sanitize expanded comment markup for safe HTML rendering.
pub fn sanitize_comment_html(body_raw: &str) -> String {
    let expanded = expand_kemoji_tokens(body_raw);
    let with_breaks = text_breaks_to_br(&expanded);
    comment_ammonia_builder().clean(&with_breaks).to_string()
}

fn comment_ammonia_builder() -> Builder<'static> {
    let mut builder = Builder::default();
    builder.add_tag_attributes("i", &["class", "data-bg", "contenteditable", "emoji"]);
    builder.add_tag_attributes("span", &["class"]);
    builder.add_tag_attributes("div", &["class"]);
    builder.attribute_filter(|element, attribute, value| {
        kemoji_attribute_filter(element, attribute, value)
    });
    builder
}

fn kemoji_attribute_filter<'a>(
    element: &str,
    attribute: &str,
    value: &'a str,
) -> Option<Cow<'a, str>> {
    match (element, attribute) {
        ("i", "class") => {
            let classes: Vec<&str> = value.split_whitespace().collect();
            if classes.len() >= 2 && classes[0] == "ke" && is_plausible_emoji_name(classes[1]) {
                let mut kept: Vec<&str> = classes
                    .into_iter()
                    .filter(|c| *c == "ke" || is_plausible_emoji_name(c))
                    .collect();
                if kept.len() < 2 {
                    return None;
                }
                kept.sort_unstable();
                kept.dedup();
                Some(kept.join(" ").into())
            } else {
                None
            }
        }
        ("i", "data-bg") => {
            if value.starts_with("url(") && !value.to_ascii_lowercase().contains("javascript:") {
                Some(value.into())
            } else {
                None
            }
        }
        ("i", "contenteditable") => Some("false".into()),
        ("i", "emoji") => {
            if is_plausible_emoji_name(value) {
                Some(value.into())
            } else {
                None
            }
        }
        ("span" | "div", "class") => {
            let safe: Vec<&str> = value
                .split_whitespace()
                .filter(|c| c.chars().all(|ch| ch.is_ascii_alphanumeric() || *c == "-"))
                .collect();
            if safe.is_empty() {
                None
            } else {
                Some(safe.join(" ").into())
            }
        }
        _ => None,
    }
}

fn kemoji_icon_html(name: &str) -> String {
    let name = name.trim();
    if !is_plausible_emoji_name(name) {
        return String::new();
    }
    let category = kemoji_category_from_name(name);
    format!(
        r#"<i class="ke {category} ke-{name}" data-bg="url('/static/fox-tpl/style/rez/{category}/emoji.png')" contenteditable="false" emoji="{name}"></i>"#,
        category = category,
        name = name
    )
}

fn kemoji_category_from_name(name: &str) -> &str {
    name.rsplit_once('_')
        .map(|(prefix, _)| prefix)
        .unwrap_or(name)
}

fn is_plausible_emoji_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn looks_like_kemoji_editor_html(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    lower.contains("ke ke-") || lower.contains(r#"class="ke"#)
}

fn strip_inline_html_tags(s: &str) -> String {
    INLINE_HTML_TAG.replace_all(s, "").into_owned()
}

fn collapse_raw_newlines(s: &str) -> String {
    s.lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn text_breaks_to_br(text: &str) -> String {
    let escaped = ammonia::clean_text(text);
    escaped.replace('\n', "<br>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_script_injection_from_prepared_html() {
        let prepared = prepare_comment_body("Hello <script>alert(1)</script> world").unwrap();
        assert!(!prepared.body_html.contains('<'));
        assert_eq!(prepared.body_raw, "Hello alert(1) world");
    }

    #[test]
    fn strips_event_handler_img_injection() {
        assert_eq!(
            prepare_comment_body(r#"<img src=x onerror=alert(1)>"#).unwrap_err(),
            CommentValidationError::BodyEmpty
        );
    }

    #[test]
    fn kemoji_text_token_renders_icon_not_raw_script() {
        let prepared = prepare_comment_body("Nice $#sperm_0DCpe#$ clip").unwrap();
        assert!(prepared.body_raw.contains("$#sperm_0DCpe#$"));
        assert!(prepared.body_html.contains(r#"class="ke"#));
        assert!(prepared.body_html.contains("ke-sperm_0DCpe"));
    }

    #[test]
    fn kemoji_html_from_editor_normalizes_to_tokens() {
        let html = r#"<img class="ke ke-one_02sXh" src="/static/style/img/opacity.png">"#;
        let text = kemoji_html_to_text(html);
        assert!(text.contains("$#one_02sXh#$"));
        let prepared = prepare_comment_body(&html).unwrap();
        assert!(prepared.body_raw.contains("$#one_02sXh#$"));
        assert!(prepared.body_html.contains("ke-one_02sXh"));
    }

    #[test]
    fn bracket_emoji_markup_expands_for_display() {
        let prepared = prepare_comment_body("Hot [sperm_0DCpe] scene").unwrap();
        assert_eq!(prepared.body_raw, "Hot [sperm_0DCpe] scene");
        assert!(prepared.body_html.contains("ke-sperm_0DCpe"));
    }

    #[test]
    fn body_raw_keeps_plain_text_separate_from_html() {
        let prepared = prepare_comment_body("  hello   world  ").unwrap();
        assert_eq!(prepared.body_raw, "hello world");
        assert_eq!(prepared.body_html, "hello world");
    }

    #[test]
    fn rejects_empty_and_oversized_bodies() {
        assert_eq!(
            normalize_body_raw("   ").unwrap_err(),
            CommentValidationError::BodyEmpty
        );
        let huge = "a".repeat(MAX_COMMENT_BODY_LEN + 1);
        assert!(matches!(
            normalize_body_raw(&huge).unwrap_err(),
            CommentValidationError::BodyTooLong { .. }
        ));
    }

    #[test]
    fn normalize_author_name_enforces_limits() {
        assert_eq!(
            normalize_author_name("  ").unwrap_err(),
            CommentValidationError::AuthorNameEmpty
        );
        let ok = normalize_author_name("  Ada  ").unwrap();
        assert_eq!(ok, "Ada");
    }
}
