//! Search-all follow-up command tests.

use serde_json::json;

use super::super::links::{canonical_search_command, top_get_command};
use super::super::plan::PreparedInput;
use super::super::{SearchAllInput, SectionKind};

#[test]
fn canonical_drug_command_stays_typed_only_with_gene_anchor() {
    let prepared = PreparedInput::new(&SearchAllInput {
        gene: Some("BRAF".to_string()),
        variant: None,
        disease: None,
        drug: None,
        keyword: Some("resistance".to_string()),
        since: None,
        limit: 3,
        counts_only: false,
        debug_plan: false,
    })
    .expect("valid prepared input");

    let command = canonical_search_command(SectionKind::Drug, &prepared, 3);
    assert!(command.contains("search drug"));
    assert!(command.contains("--target BRAF"));
    assert!(!command.contains("--query resistance"));
}

#[test]
fn canonical_article_command_keeps_keyword_only_search() {
    let prepared = PreparedInput::new(&SearchAllInput {
        gene: None,
        variant: None,
        disease: None,
        drug: None,
        keyword: Some("resistance".to_string()),
        since: None,
        limit: 3,
        counts_only: false,
        debug_plan: false,
    })
    .expect("valid prepared input");

    let command = canonical_search_command(SectionKind::Article, &prepared, 3);
    assert!(command.contains("search article"));
    assert!(command.contains("--keyword resistance"));
}

#[test]
fn canonical_article_command_dedupes_shared_disease_keyword_token() {
    let prepared = PreparedInput::new(&SearchAllInput {
        gene: None,
        variant: None,
        disease: Some("cancer".to_string()),
        drug: None,
        keyword: Some("Cancer".to_string()),
        since: None,
        limit: 3,
        counts_only: false,
        debug_plan: false,
    })
    .expect("valid prepared input");

    let command = canonical_search_command(SectionKind::Article, &prepared, 3);
    assert!(command.contains("search article"));
    assert!(!command.contains("--disease cancer"));
    assert!(command.contains("--keyword Cancer"));
}

#[test]
fn canonical_article_command_keeps_distinct_disease_and_keyword_filters() {
    let prepared = PreparedInput::new(&SearchAllInput {
        gene: None,
        variant: None,
        disease: Some("melanoma".to_string()),
        drug: None,
        keyword: Some("BRAF".to_string()),
        since: None,
        limit: 3,
        counts_only: false,
        debug_plan: false,
    })
    .expect("valid prepared input");

    let command = canonical_search_command(SectionKind::Article, &prepared, 3);
    assert!(command.contains("--disease melanoma"));
    assert!(command.contains("--keyword BRAF"));
}

#[test]
fn canonical_trial_command_stays_typed_only_with_distinct_keyword() {
    let prepared = PreparedInput::new(&SearchAllInput {
        gene: None,
        variant: None,
        disease: Some("melanoma".to_string()),
        drug: None,
        keyword: Some("BRAF".to_string()),
        since: None,
        limit: 3,
        counts_only: false,
        debug_plan: false,
    })
    .expect("valid prepared input");

    let command = canonical_search_command(SectionKind::Trial, &prepared, 3);
    assert!(command.contains("--condition melanoma"));
    assert!(!command.contains("melanoma BRAF"));
    assert!(!command.contains("--condition BRAF"));
}

#[test]
fn canonical_variant_command_preserves_unparsed_anchor() {
    let prepared = PreparedInput::new(&SearchAllInput {
        gene: None,
        variant: Some("rs121913529".to_string()),
        disease: None,
        drug: None,
        keyword: None,
        since: None,
        limit: 3,
        counts_only: false,
        debug_plan: false,
    })
    .expect("valid prepared input");

    let command = canonical_search_command(SectionKind::Variant, &prepared, 3);
    assert_eq!(command, "biomcp search variant rs121913529 --limit 3");
}

#[test]
fn canonical_article_command_quotes_apostrophe_keyword_for_shell_safety() {
    let prepared = PreparedInput::new(&SearchAllInput {
        gene: None,
        variant: None,
        disease: None,
        drug: None,
        keyword: Some("Graves'".to_string()),
        since: None,
        limit: 3,
        counts_only: false,
        debug_plan: false,
    })
    .expect("valid prepared input");

    let command = canonical_search_command(SectionKind::Article, &prepared, 3);
    assert_eq!(
        command,
        "biomcp search article --keyword \"Graves'\" --limit 3"
    );
}

#[test]
fn top_get_command_skips_civic_variant_ids() {
    let input = PreparedInput::new(&SearchAllInput {
        gene: None,
        variant: None,
        disease: None,
        drug: Some("dabrafenib".to_string()),
        keyword: None,
        since: None,
        limit: 3,
        counts_only: false,
        debug_plan: false,
    })
    .expect("valid input");
    let results = vec![
        json!({"id":"CIVIC_VARIANT:147"}),
        json!({"id":"rs113488022"}),
    ];
    let cmd = top_get_command(SectionKind::Variant, &input, &results).expect("command");
    assert_eq!(cmd, "biomcp get variant rs113488022");
}

#[test]
fn top_get_command_prefers_parent_drug_name() {
    let input = PreparedInput::new(&SearchAllInput {
        gene: None,
        variant: None,
        disease: None,
        drug: Some("dabrafenib".to_string()),
        keyword: None,
        since: None,
        limit: 3,
        counts_only: false,
        debug_plan: false,
    })
    .expect("valid input");
    let results = vec![
        json!({"name":"desmethyl dabrafenib"}),
        json!({"name":"dabrafenib"}),
    ];
    let cmd = top_get_command(SectionKind::Drug, &input, &results).expect("command");
    assert_eq!(cmd, "biomcp get drug dabrafenib");
}

#[test]
fn top_get_command_prefers_parent_like_salt_name_over_metabolites() {
    let input = PreparedInput::new(&SearchAllInput {
        gene: None,
        variant: None,
        disease: None,
        drug: Some("dabrafenib".to_string()),
        keyword: None,
        since: None,
        limit: 3,
        counts_only: false,
        debug_plan: false,
    })
    .expect("valid input");
    let results = vec![
        json!({"name":"desmethyl dabrafenib"}),
        json!({"name":"dabrafenib mesylate"}),
        json!({"name":"hydroxy dabrafenib"}),
    ];
    let cmd = top_get_command(SectionKind::Drug, &input, &results).expect("command");
    assert_eq!(cmd, "biomcp get drug dabrafenib");
}
