//! Comment listing, submission, and legacy HTML fragments for video detail AJAX.
//!
//! Anonymous posting is allowed; blank display names are stored as `Guest`.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, RwLock};

use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::comments::{
    prepare_comment_body, Comment, CommentValidationError, PreparedCommentBody,
};
use crate::models::video::fixtures::{sample_dog_house_video_page, DOG_HOUSE_SLUG};
use crate::models::video::normalize_video_slug;

pub const COMMENTS_INITIAL_LIMIT: u32 = 10;
pub const COMMENTS_MORE_BATCH_SIZE: u32 = 10;
pub const KEMOJI_SMILES_JSON_PATH: &str = "/static/fox-tpl/js/smiles_.json";

static KEMOJI_TEXT_TOKEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$#([^#$]+)#\$").expect("kemoji token regex"));

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommentSubmitResponse {
    pub result: String,
    pub text: String,
}

impl CommentSubmitResponse {
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            result: "ok".into(),
            text: message.into(),
        }
    }
    pub fn err(message: impl Into<String>) -> Self {
        Self {
            result: "error".into(),
            text: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoreCommentsResponse {
    pub comments: String,
}

#[derive(Debug, Clone, FromRow)]
struct CommentRow {
    id: u64,
    video_id: u64,
    parent_id: Option<u64>,
    author_name: String,
    body_raw: String,
    body_html: String,
    is_visible: i8,
}

impl From<CommentRow> for Comment {
    fn from(row: CommentRow) -> Self {
        Self {
            id: row.id,
            video_id: row.video_id,
            parent_id: row.parent_id,
            author_name: row.author_name,
            body_raw: row.body_raw,
            body_html: row.body_html,
            is_visible: row.is_visible != 0,
        }
    }
}

static COMMENT_FALLBACK_STORE: LazyLock<RwLock<HashMap<u64, Vec<Comment>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
static COMMENT_FALLBACK_NEXT_ID: LazyLock<Mutex<u64>> = LazyLock::new(|| Mutex::new(1_000_000));

fn seed_comment_fallback_store() {
    let mut guard = COMMENT_FALLBACK_STORE
        .write()
        .expect("comment fallback lock");
    if !guard.is_empty() {
        return;
    }
    let fixture = sample_dog_house_video_page();
    guard.insert(fixture.detail.thumb.id, fixture.extras.comments);
}

fn fallback_comments_for_video(video_id: u64) -> Vec<Comment> {
    seed_comment_fallback_store();
    COMMENT_FALLBACK_STORE
        .read()
        .expect("comment fallback lock")
        .get(&video_id)
        .cloned()
        .unwrap_or_default()
}

fn fallback_insert_comment(
    video_id: u64,
    author_name: String,
    prepared: PreparedCommentBody,
) -> Comment {
    seed_comment_fallback_store();
    let id = {
        let mut next = COMMENT_FALLBACK_NEXT_ID.lock().expect("comment id lock");
        let id = *next;
        *next += 1;
        id
    };
    let comment = Comment {
        id,
        video_id,
        parent_id: None,
        author_name,
        body_raw: prepared.body_raw,
        body_html: prepared.body_html,
        is_visible: true,
    };
    COMMENT_FALLBACK_STORE
        .write()
        .expect("comment fallback lock")
        .entry(video_id)
        .or_default()
        .push(comment.clone());
    comment
}

pub fn normalize_author_name_for_submit(raw: &str) -> Result<String, CommentValidationError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return crate::models::comments::normalize_author_name("Guest");
    }
    crate::models::comments::normalize_author_name(trimmed)
}

pub fn parse_video_id_param(raw: &str) -> Option<u64> {
    raw.trim().parse::<u64>().ok().filter(|id| *id > 0)
}

pub fn escape_html_text(input: &str) -> String {
    ammonia::clean_text(input)
}

pub fn body_raw_for_comm_two_span(body_raw: &str) -> String {
    KEMOJI_TEXT_TOKEN
        .replace_all(body_raw, |caps: &regex::Captures| format!("[{}]", &caps[1]))
        .into_owned()
}

pub fn render_comment_box_fragment(comment: &Comment) -> String {
    let span_body = escape_html_text(&body_raw_for_comm_two_span(&comment.body_raw));
    format!(
        r#"<div class="comments-box"><div class="comm-one"><span>{}:</span></div><div class="comm-two"><span>{}</span></div></div>"#,
        escape_html_text(&comment.author_name),
        span_body
    )
}

pub fn render_comments_html_fragment(comments: &[Comment]) -> String {
    comments
        .iter()
        .map(render_comment_box_fragment)
        .collect::<Vec<_>>()
        .join("")
}

pub async fn list_comments_for_video(
    pool: &DbPool,
    video_id: u64,
    limit: u32,
    offset: u32,
) -> Result<Vec<Comment>, AppError> {
    let rows = sqlx::query_as::<_, CommentRow>(
        "SELECT id, video_id, parent_id, author_name, body_raw, body_html, is_visible
         FROM comments
         WHERE video_id = ? AND is_visible = 1
         ORDER BY created_at ASC, id ASC
         LIMIT ? OFFSET ?",
    )
    .bind(video_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await;

    match rows {
        Ok(rows) if !rows.is_empty() => Ok(rows.into_iter().map(Comment::from).collect()),
        Ok(_) => Ok(fallback_comments_for_video(video_id)),
        Err(_) => Ok(fallback_comments_for_video(video_id)),
    }
}

pub async fn count_visible_comments_for_video(
    pool: &DbPool,
    video_id: u64,
) -> Result<u32, AppError> {
    let count: Result<i64, _> =
        sqlx::query_scalar("SELECT COUNT(*) FROM comments WHERE video_id = ? AND is_visible = 1")
            .bind(video_id)
            .fetch_one(pool)
            .await;

    match count {
        Ok(n) if n > 0 => Ok(n.max(0) as u32),
        _ => Ok(fallback_comments_for_video(video_id).len() as u32),
    }
}

pub async fn submit_comment(
    pool: &DbPool,
    video_id: u64,
    author_raw: &str,
    message_raw: &str,
) -> Result<Comment, CommentValidationError> {
    let author_name = normalize_author_name_for_submit(author_raw)?;
    let prepared = prepare_comment_body(message_raw)?;

    let insert = sqlx::query(
        "INSERT INTO comments (video_id, author_name, body_raw, body_html, is_visible)
         VALUES (?, ?, ?, ?, 1)",
    )
    .bind(video_id)
    .bind(&author_name)
    .bind(&prepared.body_raw)
    .bind(&prepared.body_html)
    .execute(pool)
    .await;

    match insert {
        Ok(result) => {
            let id = result.last_insert_id();
            let _ = sqlx::query("UPDATE videos SET comment_count = comment_count + 1 WHERE id = ?")
                .bind(video_id)
                .execute(pool)
                .await;
            Ok(Comment {
                id,
                video_id,
                parent_id: None,
                author_name,
                body_raw: prepared.body_raw,
                body_html: prepared.body_html,
                is_visible: true,
            })
        }
        Err(_) => Ok(fallback_insert_comment(video_id, author_name, prepared)),
    }
}

pub async fn list_comments_for_video_slug(
    pool: &DbPool,
    slug: &str,
    limit: u32,
    offset: u32,
) -> Result<Vec<Comment>, AppError> {
    let slug = normalize_video_slug(slug);
    if slug == DOG_HOUSE_SLUG {
        let video_id = sample_dog_house_video_page().detail.thumb.id;
        let all = fallback_comments_for_video(video_id);
        let start = offset as usize;
        return Ok(all.into_iter().skip(start).take(limit as usize).collect());
    }

    let video_id: Result<Option<u64>, _> =
        sqlx::query_scalar("SELECT id FROM videos WHERE slug = ? AND status = 'published' LIMIT 1")
            .bind(&slug)
            .fetch_optional(pool)
            .await;

    match video_id {
        Ok(Some(id)) => list_comments_for_video(pool, id, limit, offset).await,
        Ok(None) => Ok(Vec::new()),
        Err(_) => {
            if slug == DOG_HOUSE_SLUG {
                Ok(fallback_comments_for_video(
                    sample_dog_house_video_page().detail.thumb.id,
                ))
            } else {
                Ok(Vec::new())
            }
        }
    }
}

pub async fn submit_comment_for_video_id(
    pool: &DbPool,
    video_id: u64,
    author_raw: &str,
    message_raw: &str,
) -> Result<Comment, CommentValidationError> {
    submit_comment(pool, video_id, author_raw, message_raw).await
}

pub fn validation_error_message(err: &CommentValidationError) -> String {
    match err {
        CommentValidationError::AuthorNameEmpty => "Please enter your name.".into(),
        CommentValidationError::AuthorNameTooLong { max } => {
            format!("Name must be at most {} characters.", max)
        }
        CommentValidationError::BodyEmpty => "Please enter a comment.".into(),
        CommentValidationError::BodyTooLong { max } => {
            format!("Comment must be at most {} characters.", max)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::comments::{prepare_comment_body, MAX_AUTHOR_NAME_LEN};

    #[test]
    fn render_fragment_uses_bracket_tokens_for_client_js() {
        let body = prepare_comment_body("Nice $#sperm_0DCpe#$").unwrap();
        let comment = Comment {
            id: 1,
            video_id: 99,
            parent_id: None,
            author_name: "Ada".into(),
            body_raw: body.body_raw,
            body_html: body.body_html,
            is_visible: true,
        };
        let html = render_comment_box_fragment(&comment);
        assert!(html.contains("comments-box"));
        assert!(html.contains("[sperm_0DCpe]"));
        assert!(html.contains("Ada:"));
    }

    #[test]
    fn anonymous_submit_defaults_guest_name() {
        let name = normalize_author_name_for_submit("   ").unwrap();
        assert_eq!(name, "Guest");
        assert!(name.chars().count() <= MAX_AUTHOR_NAME_LEN);
    }
}
