//! Get-module tests split from the legacy drug facade.

use super::*;
use crate::entities::drug::interactions::{
    DrugInteractionBundleFreshness, DrugInteractionCoverageStatus, DrugInteractionFreshnessStatus,
    DrugInteractionPagination,
};
use crate::entities::drug::interactions::{LabelInteractionResult, apply_interactions_result};
use crate::entities::drug::{DrugApproval, DrugInteractionReport};
use crate::entities::section_outcome::SectionOutcomeState;

#[test]
fn parse_sections_supports_all_and_rejects_unknown() {
    let flags = parse_sections(&["all".to_string()]).unwrap();
    assert!(flags.include_label);
    assert!(flags.include_regulatory);
    assert!(flags.include_safety);
    assert!(flags.include_shortage);
    assert!(flags.include_targets);
    assert!(flags.include_indications);
    assert!(flags.include_interactions);
    assert!(flags.include_civic);
    assert!(!flags.include_approvals);

    let err = parse_sections(&["bad".to_string()]).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
}

#[test]
fn parse_sections_unknown_value_suggests_name_flag_for_multi_word_drugs() {
    let err = parse_sections_for_name(
        "tepotinib",
        &["hydrochloride".to_string(), "label".to_string()],
    )
    .unwrap_err();
    let message = err.to_string();
    assert!(message.contains("Unknown section \"hydrochloride\" for drug"));
    assert!(message.contains("--name \"tepotinib hydrochloride\" label"));
}

#[test]
fn parse_sections_all_with_explicit_label_keeps_label() {
    let flags = parse_sections(&["all".to_string(), "label".to_string()]).unwrap();
    assert!(flags.include_label);
}

#[test]
fn interaction_requests_always_permit_typed_partial_settlement() {
    assert!(
        parse_sections(&["all".to_string()])
            .unwrap()
            .allow_partial_interactions
    );
    assert!(
        parse_sections(&["label".to_string(), "interactions".to_string()])
            .unwrap()
            .allow_partial_interactions
    );
    assert!(
        parse_sections(&["interactions".to_string()])
            .unwrap()
            .allow_partial_interactions
    );
    assert!(
        parse_sections(&["interactions".to_string(), " INTERACTIONS ".to_string()])
            .unwrap()
            .allow_partial_interactions
    );
}

#[test]
fn parse_sections_default_card_includes_targets_enrichment() {
    let flags = parse_sections(&[]).unwrap();
    assert!(flags.include_targets);
}

#[test]
fn validate_region_usage_rejects_approvals_with_explicit_region() {
    let flags = parse_sections(&["approvals".to_string()]).unwrap();
    let err = validate_region_usage(&flags, DrugRegion::Us, true).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("approvals"));
}

#[test]
fn validate_region_usage_rejects_explicit_region_without_regional_sections() {
    let flags = parse_sections(&["targets".to_string()]).unwrap();
    let err = validate_region_usage(&flags, DrugRegion::Us, true).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("--region can only be used"));
}

#[test]
fn validate_region_usage_rejects_who_safety_only_requests() {
    let flags = parse_sections(&["safety".to_string()]).unwrap();
    let err = validate_region_usage(&flags, DrugRegion::Who, true).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(
        err.to_string()
            .contains("WHO regional data currently supports regulatory only")
    );
}

#[test]
fn validate_region_usage_rejects_who_shortage_only_requests() {
    let flags = parse_sections(&["shortage".to_string()]).unwrap();
    let err = validate_region_usage(&flags, DrugRegion::Who, true).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(
        err.to_string()
            .contains("WHO regional data currently supports regulatory only")
    );
}

#[test]
fn validate_region_usage_allows_who_all_requests() {
    let flags = parse_sections(&["all".to_string()]).unwrap();
    validate_region_usage(&flags, DrugRegion::Who, true).expect("who all should be valid");
}

#[test]
fn validate_raw_usage_rejects_raw_without_label_section() {
    let flags = parse_sections(&["targets".to_string()]).unwrap();
    let err = validate_raw_usage(&flags, true).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("--raw can only be used"));
}

#[test]
fn validate_raw_usage_allows_raw_with_label_section() {
    let flags = parse_sections(&["label".to_string()]).unwrap();
    validate_raw_usage(&flags, true).expect("raw label should be valid");
}

fn trial_alias(label: &str, source: TrialAliasSource) -> TrialAlias {
    TrialAlias {
        label: label.into(),
        source,
    }
}

fn trial_alias_labels(aliases: &[TrialAlias]) -> Vec<&str> {
    aliases.iter().map(|alias| alias.label.as_str()).collect()
}

#[test]
fn drugbank_trial_alias_policy_keeps_codes_and_simple_names() {
    for alias in [
        "ABT-199",
        "RMC-6236",
        "HRS 4642",
        "Venclexta",
        "O'Brien-2",
        "ééééééééééééééééééééééééééééééééé",
    ] {
        assert!(eligible_drugbank_trial_alias(alias), "should keep {alias}");
    }
}

#[test]
fn drugbank_trial_alias_policy_rejects_systematic_and_descriptor_names() {
    for alias in [
        "4-[4-[[2-(4-chlorophenyl)-4,4-dimethylcyclohex-1-enyl]methyl]piperazin-1-yl]benzoic acid",
        "ABT-199 (venetoclax free base)",
        "venetoclax free base",
        "venetoclax free   base",
        "venetoclax free\tbase",
        "venetoclax free-base",
        "ABT-199-free-base",
        "alpha,beta compound",
        "one two three four five",
    ] {
        assert!(
            !eligible_drugbank_trial_alias(alias),
            "should reject {alias}"
        );
    }
}

#[test]
fn build_trial_aliases_preserves_authorities_then_source_order_and_cap() {
    let aliases = build_trial_aliases(
        "RMC-6236",
        Some("daraxonrasib"),
        &[
            trial_alias("Zeta", TrialAliasSource::DrugBankSynonym),
            trial_alias("Venclexta", TrialAliasSource::OpenFdaBrand),
            trial_alias(" alpha ", TrialAliasSource::OpenFdaBrand),
            trial_alias("rmc-6236", TrialAliasSource::DrugBankSynonym),
            trial_alias("Beta", TrialAliasSource::DrugBankSynonym),
        ],
    );

    assert_eq!(
        trial_alias_labels(&aliases),
        vec!["RMC-6236", "daraxonrasib", "alpha", "Venclexta", "Beta"]
    );
    assert_eq!(aliases[2].source, TrialAliasSource::OpenFdaBrand);
}

#[test]
fn trial_alias_candidates_keep_untruncated_source_provenance() {
    let hit: crate::sources::mychem::MyChemHit = serde_json::from_value(serde_json::json!({
        "_id": "DB11581",
        "_score": 1.0,
        "openfda": {"brand_name": ["Venclexta", "Venclyxto"]},
        "drugbank": {"id": "DB11581", "name": "venetoclax", "synonyms": ["ABT-199", "chemical [name]"]}
    }))
    .expect("valid MyChem hit");
    let candidates = trial_alias_candidates_from_hits(&[&hit]);

    assert_eq!(candidates.len(), 4);
    assert_eq!(
        candidates[0],
        trial_alias("Venclexta", TrialAliasSource::OpenFdaBrand)
    );
    assert_eq!(
        candidates[3],
        trial_alias("chemical [name]", TrialAliasSource::DrugBankSynonym)
    );
}

#[test]
fn trial_alias_cache_key_normalizes_requested_name() {
    assert_eq!(trial_alias_cache_key(" Daraxonrasib "), "daraxonrasib");
}

#[tokio::test]
async fn cached_trial_alias_resolution_refreshes_worker_zero_label() {
    let cache_key = "ticket-510-cache-case";
    trial_alias_cache().lock().expect("cache lock").insert(
        cache_key.into(),
        TrialAliasResolution {
            canonical_name: "canonical".into(),
            aliases: vec![trial_alias(
                "Ticket-510-Cache-Case",
                TrialAliasSource::Requested,
            )],
        },
    );

    let resolution = resolve_trial_alias_resolution("ticket-510-cache-case")
        .await
        .expect("cached resolution");
    assert_eq!(resolution.aliases[0].label, "ticket-510-cache-case");
    trial_alias_cache()
        .lock()
        .expect("cache lock")
        .remove(cache_key);
}

#[test]
fn trial_alias_resolution_does_not_cache_transient_lookup_failure() {
    let requested = "review-transient-alias-drug";
    let (fallback, fallback_cacheable) = trial_alias_resolution_from_lookup_result(
        requested,
        Err(BioMcpError::Api {
            api: "mychem.info".into(),
            message: "HTTP 500".into(),
        }),
    );
    assert_eq!(trial_alias_labels(&fallback.aliases), vec![requested]);
    assert!(!fallback_cacheable);

    let (resolved, resolved_cacheable) = trial_alias_resolution_from_lookup_result(
        requested,
        Ok(TrialAliasLookup {
            canonical_name: requested.into(),
            candidates: vec![trial_alias("RMC-6236", TrialAliasSource::DrugBankSynonym)],
        }),
    );
    assert_eq!(
        trial_alias_labels(&resolved.aliases),
        vec![requested, "RMC-6236"]
    );
    assert!(resolved_cacheable);
}

#[test]
fn trial_alias_resolution_keeps_generic_requests_canonical() {
    let requested = "pembrolizumab";
    let (resolved, cacheable) = trial_alias_resolution_from_lookup_result(
        requested,
        Ok(TrialAliasLookup {
            canonical_name: requested.into(),
            candidates: vec![trial_alias("Keytruda", TrialAliasSource::OpenFdaBrand)],
        }),
    );

    assert_eq!(resolved.canonical_name, requested);
    assert_eq!(
        trial_alias_labels(&resolved.aliases),
        vec!["pembrolizumab", "Keytruda"]
    );
    assert!(cacheable);
}

fn test_approval_drug() -> Drug {
    crate::transform::drug::merge_mychem_hits(&[], "fixture-drug")
}

fn interaction_report(
    rows: Vec<super::super::DrugInteraction>,
    label: Option<&str>,
) -> DrugInteractionReport {
    let count = rows.len();
    DrugInteractionReport {
        name: "fixture-drug".to_string(),
        drugbank_id: Some("DB00000".to_string()),
        chembl_id: None,
        interactions: rows,
        pagination: DrugInteractionPagination {
            total: count,
            count,
            offset: 0,
            limit: 25,
            next_command: None,
        },
        bundle_freshness: DrugInteractionBundleFreshness {
            status: DrugInteractionFreshnessStatus::Fresh,
        },
        coverage_status: DrugInteractionCoverageStatus::InDdinterCoverage,
        source_note: None,
        coverage_note: None,
        label_interaction_text: label.map(str::to_string),
    }
}

fn interaction_row(description: Option<&str>) -> super::super::DrugInteraction {
    super::super::DrugInteraction {
        drug: "aspirin".to_string(),
        ddinter_id: Some("DDInter1".to_string()),
        level: Some("Major".to_string()),
        description: description.map(str::to_string),
        partner_classes: Vec::new(),
    }
}

fn interaction_failure(kind: &str) -> BioMcpError {
    match kind {
        "connection" => BioMcpError::Api {
            api: "DDInter".to_string(),
            message: "SENSITIVE-UPSTREAM-DETAIL connection refused".to_string(),
        },
        "timeout" => BioMcpError::SourceUnavailable {
            source_name: "DDInter".to_string(),
            reason: "SENSITIVE-UPSTREAM-DETAIL request timed out".to_string(),
            suggestion: "retry".to_string(),
        },
        "malformed" => BioMcpError::ApiJson {
            api: "DDInter".to_string(),
            source: serde_json::from_str::<serde_json::Value>("{")
                .expect_err("fixture body must be malformed"),
        },
        other => panic!("unknown failure {other}"),
    }
}

fn assert_interaction_outcome(drug: &Drug, state: SectionOutcomeState, sources: &[&str]) {
    let outcome = drug
        .section_outcomes
        .get("interactions")
        .expect("registered interaction outcome");
    assert_eq!(outcome.outcome(), state);
    assert_eq!(
        outcome.sources(),
        sources
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
    );
}

#[test]
fn interaction_failure_preserves_only_surviving_label_evidence_without_leaking_errors() {
    for kind in ["connection", "timeout", "malformed"] {
        let mut with_label = test_approval_drug();
        apply_interactions_result(
            &mut with_label,
            Err(interaction_failure(kind)),
            LabelInteractionResult::Data("Warfarin precaution".to_string()),
        );
        assert!(with_label.interactions.is_empty());
        assert_eq!(
            with_label.interaction_text.as_deref(),
            Some("Warfarin precaution")
        );
        assert!(with_label.interaction_pagination.is_none());
        assert!(with_label.interaction_bundle_freshness.is_none());
        assert_interaction_outcome(
            &with_label,
            SectionOutcomeState::Degraded,
            &["OpenFDA label"],
        );
        assert_eq!(
            crate::render::provenance::drug_interaction_heading_label(&with_label),
            "Interactions"
        );
        assert!(
            !serde_json::to_string(&with_label)
                .unwrap()
                .contains("SENSITIVE-UPSTREAM-DETAIL")
        );

        let mut without_label = test_approval_drug();
        apply_interactions_result(
            &mut without_label,
            Err(interaction_failure(kind)),
            LabelInteractionResult::Empty,
        );
        assert_interaction_outcome(&without_label, SectionOutcomeState::Unavailable, &[]);
    }
}

#[test]
fn interaction_additive_source_state_matrix_is_truthful() {
    let cases = [
        (
            Some(vec![interaction_row(None)]),
            LabelInteractionResult::Unavailable,
            SectionOutcomeState::Degraded,
            vec!["DDInter"],
        ),
        (
            Some(vec![interaction_row(Some("DrugBank narrative"))]),
            LabelInteractionResult::Unavailable,
            SectionOutcomeState::Degraded,
            vec!["DDInter", "DrugBank"],
        ),
        (
            Some(Vec::new()),
            LabelInteractionResult::Unavailable,
            SectionOutcomeState::Unavailable,
            vec![],
        ),
        (
            None,
            LabelInteractionResult::Data("Label evidence".to_string()),
            SectionOutcomeState::Degraded,
            vec!["OpenFDA label"],
        ),
        (
            None,
            LabelInteractionResult::Empty,
            SectionOutcomeState::Unavailable,
            vec![],
        ),
        (
            None,
            LabelInteractionResult::Unavailable,
            SectionOutcomeState::Unavailable,
            vec![],
        ),
        (
            Some(vec![interaction_row(None)]),
            LabelInteractionResult::Empty,
            SectionOutcomeState::Data,
            vec!["DDInter"],
        ),
        (
            Some(vec![interaction_row(Some("DrugBank narrative"))]),
            LabelInteractionResult::Empty,
            SectionOutcomeState::Data,
            vec!["DDInter", "DrugBank"],
        ),
        (
            Some(vec![interaction_row(None)]),
            LabelInteractionResult::Data("Label evidence".to_string()),
            SectionOutcomeState::Data,
            vec!["DDInter", "OpenFDA label"],
        ),
        (
            Some(vec![interaction_row(Some("DrugBank narrative"))]),
            LabelInteractionResult::Data("Label evidence".to_string()),
            SectionOutcomeState::Data,
            vec!["DDInter", "DrugBank", "OpenFDA label"],
        ),
        (
            Some(Vec::new()),
            LabelInteractionResult::Data("Label evidence".to_string()),
            SectionOutcomeState::Data,
            vec!["OpenFDA label"],
        ),
        (
            Some(Vec::new()),
            LabelInteractionResult::Empty,
            SectionOutcomeState::Empty,
            vec!["DDInter", "OpenFDA label"],
        ),
    ];

    for (rows, label, state, sources) in cases {
        let mut drug = test_approval_drug();
        let ddinter_failed = rows.is_none();
        let result = rows.map_or_else(
            || Err(interaction_failure("timeout")),
            |rows| Ok(interaction_report(rows, None)),
        );
        apply_interactions_result(&mut drug, result, label);
        assert_interaction_outcome(&drug, state, &sources);
        let outcome = drug.section_outcomes.get("interactions").unwrap();
        assert_eq!(
            outcome.message(),
            match state {
                SectionOutcomeState::Degraded => Some(
                    "Drug interaction evidence is incomplete because a source was unavailable."
                ),
                SectionOutcomeState::Unavailable => {
                    Some("Drug interaction evidence is temporarily unavailable.")
                }
                _ => None,
            }
        );
        if ddinter_failed {
            assert!(drug.interactions.is_empty());
            assert!(drug.interaction_pagination.is_none());
            assert!(drug.interaction_bundle_freshness.is_none());
        }
        assert_eq!(
            crate::render::provenance::drug_interaction_heading_label(&drug),
            if sources.contains(&"DDInter") {
                "Interactions (DDInter)"
            } else {
                "Interactions"
            }
        );
    }
}

fn injected_approval_failure(kind: &str) -> BioMcpError {
    match kind {
        "connection-refused" => BioMcpError::Api {
            api: "OpenFDA".to_string(),
            message: "connection refused".to_string(),
        },
        "timeout" => BioMcpError::SourceUnavailable {
            source_name: "OpenFDA".to_string(),
            reason: "request timed out".to_string(),
            suggestion: "retry".to_string(),
        },
        "malformed-body" => BioMcpError::ApiJson {
            api: "OpenFDA".to_string(),
            source: serde_json::from_str::<serde_json::Value>("{")
                .expect_err("fixture body must be malformed"),
        },
        other => panic!("unknown injected failure: {other}"),
    }
}

fn assert_approval_outcome(
    drug: &Drug,
    expected: crate::entities::section_outcome::SectionOutcomeState,
) {
    use crate::entities::section_outcome::SectionOutcomeState;

    let outcome = drug
        .section_outcomes
        .get("approvals")
        .expect("registered approvals outcome");
    assert_eq!(outcome.outcome(), expected);
    if expected == SectionOutcomeState::Unavailable {
        assert!(outcome.sources().is_empty());
    } else {
        assert_eq!(outcome.sources(), &["OpenFDA Drugs@FDA".to_string()]);
    }
}

#[test]
fn drugsfda_failure_state_matrix() {
    use crate::entities::section_outcome::SectionOutcomeState;

    for failure in ["connection-refused", "timeout", "malformed-body"] {
        let mut drug = test_approval_drug();
        let error = injected_approval_failure(failure);
        let private_detail = error.to_string();
        apply_approvals_result(&mut drug, Err::<Vec<DrugApproval>, _>(error));
        assert!(
            drug.approvals
                .as_ref()
                .expect("compatibility array")
                .is_empty()
        );
        assert_approval_outcome(&drug, SectionOutcomeState::Unavailable);
        assert!(
            !serde_json::to_string(&drug)
                .expect("failed approvals state serializes")
                .contains(&private_detail)
        );
    }

    let mut empty = test_approval_drug();
    apply_approvals_result(&mut empty, Ok(Vec::new()));
    assert!(
        empty
            .approvals
            .as_ref()
            .expect("healthy empty array")
            .is_empty()
    );
    assert_approval_outcome(&empty, SectionOutcomeState::Empty);

    let mut data = test_approval_drug();
    apply_approvals_result(
        &mut data,
        Ok(vec![DrugApproval {
            application_number: "NDA021304".to_string(),
            sponsor_name: Some("Example Pharma".to_string()),
            openfda_brand_names: vec!["DrugX".to_string()],
            openfda_generic_names: vec!["drugx".to_string()],
            products: Vec::new(),
            submissions: Vec::new(),
        }]),
    );
    assert_eq!(
        data.approvals.as_ref().expect("approval payload")[0].application_number,
        "NDA021304"
    );
    assert_approval_outcome(&data, SectionOutcomeState::Data);
}
