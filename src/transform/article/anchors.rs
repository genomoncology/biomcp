//! Text cleanup and truncation helpers for article transforms.

use std::sync::OnceLock;

use regex::Regex;

use super::collapse_whitespace;

fn truncate_utf8(s: &str, max_bytes: usize, suffix: &str) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }

    let mut boundary = max_bytes;
    while boundary > 0 && !s.is_char_boundary(boundary) {
        boundary -= 1;
    }
    let mut out = s[..boundary].trim_end().to_string();
    out.push_str(suffix);
    out
}

fn decode_html_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}

fn strip_inline_html_tags(value: &str) -> String {
    static HTML_TAG_RE: OnceLock<Regex> = OnceLock::new();
    let re = HTML_TAG_RE.get_or_init(|| Regex::new(r"(?is)<[^>]+>").expect("valid regex"));
    re.replace_all(value, "").to_string()
}

fn normalize_compound_hyphens(value: &str) -> String {
    static COMPOUND_HYPHEN_RE: OnceLock<Regex> = OnceLock::new();
    let re = COMPOUND_HYPHEN_RE
        .get_or_init(|| Regex::new(r"([a-z])-(\d)").expect("valid compound-hyphen regex"));
    re.replace_all(value, "${1}${2}").into_owned()
}

pub fn clean_title(value: &str) -> String {
    strip_inline_html_tags(&decode_html_entities(value))
        .trim()
        .to_string()
}

pub fn clean_abstract(value: &str) -> String {
    strip_inline_html_tags(&decode_html_entities(value))
        .trim()
        .to_string()
}

pub fn normalize_article_search_text(value: &str) -> String {
    let base = collapse_whitespace(&clean_abstract(value)).to_ascii_lowercase();
    if !base.contains('-') {
        return base;
    }
    normalize_compound_hyphens(&base)
}

fn truncate_title(title: &str) -> String {
    const MAX_TITLE_BYTES: usize = 60;
    truncate_utf8(&clean_title(title), MAX_TITLE_BYTES, "…")
}

pub fn article_search_fallback_title(text: &str) -> String {
    truncate_title(text)
}

pub fn truncate_abstract(text: &str) -> String {
    const MAX_ABSTRACT_BYTES: usize = 1500;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.len() <= MAX_ABSTRACT_BYTES {
        return trimmed.to_string();
    }

    let short = truncate_utf8(trimmed, MAX_ABSTRACT_BYTES, "...");
    let total = trimmed.chars().count();
    format!("{short}\n\n(truncated, {total} chars total)")
}

pub fn article_search_abstract_snippet(text: &str) -> Option<String> {
    const MAX_ABSTRACT_BYTES: usize = 240;
    let cleaned = clean_abstract(text);
    if cleaned.is_empty() {
        return None;
    }
    let snippet = if cleaned.len() <= MAX_ABSTRACT_BYTES {
        cleaned
    } else {
        truncate_utf8(&cleaned, MAX_ABSTRACT_BYTES, "...")
    };
    Some(snippet)
}

#[cfg(test)]
mod tests;
