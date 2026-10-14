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

#[test]
fn orphan_aliases_require_matching_nonconflicting_anchor() {
    let drug = test_approval_drug();
    let drug = Drug {
        drugbank_id: Some("DB1".into()),
        chembl_id: Some("CHEMBL1".into()),
        unii: None,
        name: "canonical".into(),
        ..drug
    };
    let matching: MyChemHit = serde_json::from_value(serde_json::json!({
        "drugbank": {"id":"DB1", "name":"Salt qualified", "synonyms":["Alias"]},
        "chembl": {"molecule_chembl_id":"CHEMBL1", "pref_name":"Preferred"},
        "openfda": {"generic_name":["Generic"], "brand_name":["Brand"]}
    }))
    .unwrap();
    let conflicting: MyChemHit = serde_json::from_value(serde_json::json!({
        "drugbank": {"id":"DB1", "name":"Wrong alias"},
        "chembl": {"molecule_chembl_id":"CHEMBL2", "pref_name":"Conflict"}
    }))
    .unwrap();
    assert_eq!(
        orphan_aliases(" requested ", &drug, &[matching, conflicting]),
        vec![
            "requested",
            "canonical",
            "Salt qualified",
            "Alias",
            "Preferred",
            "Generic"
        ]
    );
}

#[test]
fn orphan_aliases_do_not_trust_anchorless_hits() {
    let drug = test_approval_drug();
    let hit: MyChemHit = serde_json::from_value(
        serde_json::json!({"drugbank":{"name":"Unowned", "synonyms":["Alias"]}}),
    )
    .unwrap();
    assert_eq!(
        orphan_aliases("requested", &drug, &[hit]),
        vec!["requested", "fixture-drug"]
    );
}

#[test]
fn orphan_aliases_treat_an_absent_identifier_as_neutral() {
    let drug = Drug {
        drugbank_id: Some("DB1".into()),
        chembl_id: Some("CHEMBL1".into()),
        name: "canonical".into(),
        ..test_approval_drug()
    };
    let hit: MyChemHit = serde_json::from_value(serde_json::json!({
        "drugbank": {"id":"DB1", "name":"Partially populated salt"}
    }))
    .unwrap();
    assert_eq!(
        orphan_aliases("requested", &drug, &[hit]),
        vec!["requested", "canonical", "Partially populated salt"]
    );
}

#[test]
fn orphan_aliases_reject_a_multi_unii_hit_with_any_conflict() {
    let drug = Drug {
        unii: Some("MATCH".into()),
        name: "canonical".into(),
        ..test_approval_drug()
    };
    let hit: MyChemHit = serde_json::from_value(serde_json::json!({
        "unii": [{"unii":"MATCH", "display_name":"Looks owned"}, {"unii":"CONFLICT"}],
        "openfda": {"brand_name":["Rejected brand"]}
    }))
    .unwrap();
    assert_eq!(
        orphan_aliases("requested", &drug, &[hit]),
        vec!["requested", "canonical"]
    );
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

#[test]
fn unreadable_who_bundle_degrades_the_card_section_without_a_local_path() {
    // With WHO broken, `get drug <name>
    // regulatory` must not error outright carrying a local data path. The
    // card degrades: the WHO section is omitted with a note that names the
    // gap, and the note carries no local path, so JSON and MCP callers
    // never see one.
    let unavailable = BioMcpError::SourceUnavailable {
        source_name: "WHO Prequalification".to_string(),
        reason: format!(
            "{} Refresh failed for who_pq.csv: WHO Prequalification export headers did not match: who_pq.csv: missing required column BASIS OF LISTING. Missing required WHO Prequalification file(s) after the run: who_pq.csv.",
            crate::sources::who_pq::WHO_PQ_SYNC_FAILURE_REASON_PREFIX
        ),
        suggestion: crate::sources::who_pq::who_pq_sync_failure_recovery(std::path::Path::new(
            "/home/who/private-data",
        )),
    };
    let mut drug = test_approval_drug();
    apply_who_sections_result(&mut drug, "zidovudine", Err(unavailable)).expect("degrade");
    assert!(drug.who_prequalification.is_none());
    let note = drug
        .who_note
        .as_deref()
        .expect("degraded card carries the note");
    assert!(note.contains("WHO Prequalification data is unavailable"));
    assert!(note.contains("so this card omits the WHO section"));
    assert!(note.contains("biomcp who sync"));
    assert!(note.contains("BIOMCP_WHO_DIR"));
    assert!(
        !note.contains("/home/who"),
        "the card note must stay path-free: {note}"
    );
    // The JSON surface (and so MCP `get drug ... regulatory` with --json)
    // carries the same note and no path.
    let json = serde_json::to_value(&drug).expect("drug JSON");
    assert_eq!(
        json["who_note"].as_str(),
        Some(note),
        "the degraded note must reach JSON"
    );
    assert!(!json.to_string().contains("/home/who"));

    // A readable bundle clears the note and fills the section.
    let root = crate::test_support::TempDirGuard::new("who-card-ready");
    let fixture_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("spec")
        .join("fixtures")
        .join("who-pq");
    for file in crate::sources::who_pq::WHO_PQ_REQUIRED_FILES {
        std::fs::copy(fixture_dir.join(file), root.path().join(file)).expect("copy WHO fixture");
    }
    let mut drug = test_approval_drug();
    drug.who_note = Some("stale note".into());
    apply_who_sections_result(
        &mut drug,
        "zidovudine",
        Ok(crate::sources::who_pq::WhoPqClient::from_root(root.path())),
    )
    .expect("populated bundle root resolves sections");
    assert!(drug.who_note.is_none());
    assert!(drug.who_prequalification.is_some());
}

/// The openFDA label answer for `Pain Relief` (live total 1249, recorded
/// 2026-10-14, minimized to the identity fields the resolution reads):
/// five newest records whose brands merely contain the phrase, each
/// carrying a different single ingredient. The full capture and its
/// receipt live at `testdata/sources/openfda/label_pain_relief_20261014.json`.
const PAIN_RELIEF_LABEL_BODY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/testdata/sources/openfda/label_pain_relief_20261014.json"
));

fn label_page_results(body: &str) -> Vec<serde_json::Value> {
    serde_json::from_str::<serde_json::Value>(body)
        .expect("label body parses")
        .get("results")
        .and_then(|value| value.as_array())
        .expect("label body holds results")
        .clone()
}

fn synthetic_label_record(brand: &str, generic: &str, substances: &[&str]) -> serde_json::Value {
    serde_json::json!({
        "set_id": format!("synthetic-{brand}"),
        "effective_time": "20260901",
        "openfda": {
            "brand_name": [brand],
            "generic_name": [generic],
            "substance_name": substances,
        }
    })
}

#[test]
fn a_disagreeing_label_page_refuses_the_name_as_ambiguous() {
    let answer = label_identity_answer("Pain Relief", &label_page_results(PAIN_RELIEF_LABEL_BODY));
    let LabelIdentityAnswer::Ambiguous(candidates) = answer else {
        panic!("the recorded Pain Relief page must refuse as ambiguous");
    };
    // Lidocaine and menthol each answer on two of the five records;
    // acetaminophen and hexylresorcinol on one: record count first, then
    // name.
    let listed = candidates
        .iter()
        .map(|candidate| (candidate.records, candidate.name.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        listed,
        [
            (2, "lidocaine"),
            (2, "menthol"),
            (1, "acetaminophen"),
            (1, "hexylresorcinol"),
        ]
    );
    assert_eq!(
        ambiguous_drug_name_error("Pain Relief", &candidates).to_string(),
        "Invalid argument: Ambiguous drug name 'Pain Relief': several drug products carry this name.\n\
Candidates:\n- lidocaine\n- menthol\n- acetaminophen\n- hexylresorcinol\n\
Try `biomcp get drug \"<ingredient>\"`"
    );
}

#[test]
fn ambiguous_candidates_come_from_the_records_that_own_the_name() {
    // The recorded "Sleep Aid" shape (live total 247, recorded 2026-10-14):
    // two owner records carry two distinct single ingredients while a
    // third record merely holds the phrase in a longer brand.
    let page = vec![
        synthetic_label_record(
            "Sleep Aid",
            "DIPHENHYDRAMINE HCL",
            &["DIPHENHYDRAMINE HYDROCHLORIDE"],
        ),
        synthetic_label_record(
            "SLEEP AID",
            "DOXYLAMINE SUCCINATE",
            &["DOXYLAMINE SUCCINATE"],
        ),
        synthetic_label_record(
            "Nighttime Sleep Aid Berry Flavor",
            "DIPHENHYDRAMINE HYDROCHLORIDE",
            &["DIPHENHYDRAMINE HYDROCHLORIDE"],
        ),
    ];
    let answer = label_identity_answer("Sleep Aid", &page);
    let LabelIdentityAnswer::Ambiguous(candidates) = answer else {
        panic!("the Sleep Aid page must refuse as ambiguous");
    };
    // Only the owner records' ingredients count, so the third record adds
    // no diphenhydramine count and no third candidate.
    assert_eq!(
        candidates
            .iter()
            .map(|candidate| (candidate.records, candidate.name.as_str()))
            .collect::<Vec<_>>(),
        [
            (1, "diphenhydramine hydrochloride"),
            (1, "doxylamine succinate")
        ]
    );
}

#[test]
fn ambiguous_candidates_rank_by_record_count_then_name_and_cap_at_ten() {
    let seconds = [
        "CHLORPHENIRAMINE MALEATE",
        "DEXTROMETHORPHAN HBR",
        "DOXYLAMINE SUCCINATE",
        "PHENYLEPHRINE HCL",
        "TRIPROLIDINE HCL",
        "BROMPHENIRAMINE MALEATE",
        "PSEUDOEPHEDRINE HCL",
        "GUAIFENESIN",
        "IBUPROFEN",
        "NAPROXEN SODIUM",
        "CETIRIZINE HCL",
    ];
    let page = seconds
        .iter()
        .map(|second| {
            synthetic_label_record(
                "Cold and Flu",
                &format!("ACETAMINOPHEN, {second}"),
                &["ACETAMINOPHEN", second],
            )
        })
        .collect::<Vec<_>>();
    let answer = label_identity_answer("Cold and Flu", &page);
    let LabelIdentityAnswer::Ambiguous(candidates) = answer else {
        panic!("the Cold and Flu page must refuse as ambiguous");
    };
    // Acetaminophen answers on every record, so it leads; the rest rank
    // by name, and the cap keeps the first ten lines.
    assert_eq!(
        candidates
            .iter()
            .map(|candidate| (candidate.records, candidate.name.as_str()))
            .collect::<Vec<_>>(),
        [
            (11, "acetaminophen"),
            (1, "brompheniramine maleate"),
            (1, "cetirizine hcl"),
            (1, "chlorpheniramine maleate"),
            (1, "dextromethorphan hbr"),
            (1, "doxylamine succinate"),
            (1, "guaifenesin"),
            (1, "ibuprofen"),
            (1, "naproxen sodium"),
            (1, "phenylephrine hcl"),
        ]
    );
}

/// The recorded-reply proof for the ticket: the live openFDA answer for
/// "Pain Relief", replayed through the fixture server, refuses the name
/// as ambiguous with exit 2 and the ingredient candidates, before the
/// discover rescue can answer it as a symptom.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn get_drug_pain_relief_refuses_as_ambiguous_with_the_recorded_candidates() {
    let (base, server) = crate::entities::drug::test_support::name_resolution_fixture_server(
        // The live MyChem text query matches only other drugs' records
        // (live total 673, trimmed to one row).
        vec![(
            "Pain Relief".to_string(),
            r#"{"total":673,"hits":[{"_id":"RZVAJINKPMORJF-UHFFFAOYSA-N","_score":21.7,"ndc":{"nonproprietaryname":"Acetaminophen, Caffeine, Pyrilamine maleate","proprietaryname":"FLO PMS Complete"}}]}"#
                .to_string(),
        )],
        vec![(
            "Pain Relief".to_string(),
            PAIN_RELIEF_LABEL_BODY.to_string(),
        )],
        Vec::new(),
    )
    .await;
    let (_env, _ddinter) =
        crate::entities::drug::test_support::drug_fixture_env(&base, "pain-relief-ddinter");

    let err = super::get("Pain Relief", &["label".to_string()])
        .await
        .expect_err("a name several products share must refuse");
    assert_eq!(err.exit_code(), 2, "{err}");
    assert_eq!(
        err.to_string(),
        "Invalid argument: Ambiguous drug name 'Pain Relief': several drug products carry this name.\n\
Candidates:\n- lidocaine\n- menthol\n- acetaminophen\n- hexylresorcinol\n\
Try `biomcp get drug \"<ingredient>\"`"
    );
    server.abort();
}
