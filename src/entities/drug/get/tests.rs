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
