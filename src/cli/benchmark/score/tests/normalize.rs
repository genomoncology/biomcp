//! Score-session command normalization tests.

use super::*;

#[test]
fn normalize_command_shape_tracks_structure() {
    let shape = normalize_command_shape("biomcp search article -g BRAF --limit 5")
        .expect("shape should parse");
    assert_eq!(shape, "search article -g <value> --limit <value>");

    let shape = normalize_command_shape("bash -lc 'biomcp get gene BRAF pathways'")
        .expect("shape should parse");
    assert_eq!(shape, "get gene <arg> pathways");
}

#[test]
fn returns_none_for_non_biomcp_commands() {
    assert!(normalize_command_shape("echo hello").is_none());
}

#[test]
fn section_like_tokens_include_new_gene_enrichment_sections() {
    assert!(is_section_like_token("expression"));
    assert!(is_section_like_token("hpa"));
    assert!(is_section_like_token("druggability"));
    assert!(is_section_like_token("clingen"));
    assert!(is_section_like_token("constraint"));
    assert!(is_section_like_token("disgenet"));
}
