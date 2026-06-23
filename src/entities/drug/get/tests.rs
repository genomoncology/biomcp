//! Get-module tests split from the legacy drug facade.

use super::*;

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

#[test]
fn trial_alias_filter_rejects_formulation_strength_variants() {
    assert!(looks_like_trial_formulation_variant("Keytruda 25 mg/mL"));
    assert!(looks_like_trial_formulation_variant(
        "Pembrolizumab injection"
    ));
}

#[test]
fn trial_alias_filter_keeps_sponsor_codes() {
    assert!(!looks_like_trial_formulation_variant("RMC-6236"));
}

#[test]
fn build_trial_aliases_preserves_requested_canonical_and_brand_order() {
    let aliases = build_trial_aliases(
        "RMC-6236",
        Some("daraxonrasib"),
        &[
            "RMC-6236".to_string(),
            "Keytruda 25 mg/mL".to_string(),
            "RMC-6236".to_string(),
            "daraxonrasib".to_string(),
            "RMC-9805".to_string(),
        ],
    );

    assert_eq!(aliases, vec!["RMC-6236", "daraxonrasib", "RMC-9805"]);
}

#[test]
fn trial_alias_cache_key_normalizes_requested_name() {
    assert_eq!(trial_alias_cache_key(" Daraxonrasib "), "daraxonrasib");
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
    assert_eq!(fallback.aliases, vec![requested.to_string()]);
    assert!(!fallback_cacheable);

    let (resolved, resolved_cacheable) = trial_alias_resolution_from_lookup_result(
        requested,
        Ok(TrialAliasLookup {
            canonical_name: requested.into(),
            brand_names: vec!["RMC-6236".into()],
        }),
    );
    assert_eq!(
        resolved.aliases,
        vec![requested.to_string(), "RMC-6236".to_string()]
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
            brand_names: vec!["Keytruda".into()],
        }),
    );

    assert_eq!(resolved.canonical_name, requested);
    assert_eq!(resolved.aliases, vec!["pembrolizumab", "Keytruda"]);
    assert!(cacheable);
}
