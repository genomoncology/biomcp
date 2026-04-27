//! Routing tests for the `biomcp list` facade.

use super::super::render;

#[test]
fn list_skill_alias_routes_to_skill_listing() {
    let out = render(Some("skill")).expect("list skill should render");
    assert!(out.contains("# BioMCP Worked Examples"));
    assert!(out.contains("01 treatment-lookup"));
    assert!(out.contains("04 article-follow-up"));
    assert!(out.contains("15 negative-evidence"));
}

#[test]
fn unknown_entity_lists_new_valid_entities() {
    let err = render(Some("unknown")).expect_err("unknown entity should fail");
    let msg = err.to_string();
    assert!(msg.contains("- skill"));
    assert!(msg.contains("- enrich"));
    assert!(msg.contains("- batch"));
    assert!(msg.contains("- study"));
    assert!(msg.contains("- suggest"));
    assert!(msg.contains("- discover"));
}
