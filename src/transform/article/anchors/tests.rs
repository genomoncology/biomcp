//! Anchor and search-text regression tests.

use super::*;

#[test]
fn truncate_title_truncates_on_utf8_boundary() {
    let title = "€".repeat(100);
    let out = truncate_title(&title);
    assert!(out.ends_with('…'));
    assert!(out.len() <= 63);
}

#[test]
fn truncate_title_strips_inline_html_and_entities() {
    let title = "KRAS&lt;sup&gt;G12C&lt;/sup&gt; and <i>melanoma</i>";
    let out = truncate_title(title);
    assert!(out.contains("KRAS"));
    assert!(!out.contains("&lt;"));
    assert!(!out.contains("<i>"));
}

#[test]
fn normalize_article_search_text_compacts_compound_hyphens() {
    assert_eq!(normalize_article_search_text("LB-100"), "lb100");
    assert_eq!(normalize_article_search_text("LB100"), "lb100");
    assert_eq!(normalize_article_search_text("IL-2"), "il2");
    assert_eq!(
        normalize_article_search_text("meta-analysis"),
        "meta-analysis"
    );
}

#[test]
fn truncate_abstract_keeps_full_text_until_limit() {
    let text = "Sentence one. Sentence two. Sentence three.";
    let out = truncate_abstract(text);
    assert_eq!(out, text);
}
