//! Search query normalization and display strings for `/videos/{term}` pages.

/// Minimum characters for a search term (matches live JS submit guard).
pub const SEARCH_MIN_QUERY_LEN: usize = 3;

/// Slugify a raw search query for canonical `/videos/{slug}` paths.
pub fn slugify_search_query(raw: &str) -> String {
    let lower = raw.trim().to_ascii_lowercase();
    let mut out = String::new();
    let mut last_hyphen = false;
    for ch in lower.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_hyphen = false;
        } else if !last_hyphen && !out.is_empty() {
            out.push('-');
            last_hyphen = true;
        }
    }
    out.trim_matches('-').to_string()
}

/// Title-case display label from a path slug (`test` → `Test`, `big-tits` → `Big Tits`).
pub fn display_term_from_slug(slug: &str) -> String {
    let trimmed = slug.trim().trim_matches('-');
    if trimmed.is_empty() {
        return String::new();
    }
    trimmed
        .split('-')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let mut word = String::new();
                    word.push(first.to_ascii_uppercase());
                    word.push_str(chars.as_str());
                    word
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn search_h1(display_term: &str) -> String {
    format!("{display_term} Porn Videos")
}

pub fn search_page_title(display_term: &str) -> String {
    format!("{display_term} Porn Videos & Sex Scenes | PornsOK.com")
}

pub fn search_meta_description(display_term: &str) -> String {
    let lower = display_term.to_ascii_lowercase();
    format!(
        "The best {lower} porn can be found for free on PornsOK. If you're obsessed with {lower} sex videos ⏩, you have come to the right porn tube."
    )
}

pub fn normalize_search_needle(input: &str) -> String {
    input.trim().replace('%', "").replace('_', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_trims_and_hyphenates() {
        assert_eq!(slugify_search_query("  Test  "), "test");
        assert_eq!(slugify_search_query("Big Tits"), "big-tits");
    }

    #[test]
    fn display_term_from_slug_title_cases() {
        assert_eq!(display_term_from_slug("test"), "Test");
        assert_eq!(display_term_from_slug("big-tits"), "Big Tits");
    }
}
