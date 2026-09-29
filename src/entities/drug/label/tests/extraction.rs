use super::*;

#[test]
fn extract_interaction_text_from_label_uses_openfda_drug_interactions() {
    let response = serde_json::json!({
        "results": [{
            "drug_interactions": [
                "DRUG INTERACTIONS",
                "Warfarin has documented interactions with aspirin."
            ]
        }]
    });

    let text = extract_interaction_text_from_label(&response).expect("interaction text");
    assert!(text.contains("DRUG INTERACTIONS"));
    assert!(text.contains("Warfarin has documented interactions with aspirin."));
}

#[test]
fn extract_interaction_text_from_label_returns_none_when_missing() {
    let response = serde_json::json!({
        "results": [{
            "warnings_and_cautions": ["No interaction section present"]
        }]
    });

    assert_eq!(extract_interaction_text_from_label(&response), None);
}

#[test]
fn extract_label_set_id_prefers_top_level_set_id() {
    let response = serde_json::json!({
        "results": [{
            "set_id": "abc-123",
            "openfda": {
                "spl_set_id": ["fallback-456"]
            }
        }]
    });

    assert_eq!(extract_label_set_id(&response).as_deref(), Some("abc-123"));
}

#[test]
fn extract_label_set_id_falls_back_to_spl_set_id() {
    let response = serde_json::json!({
        "results": [{
            "openfda": {
                "spl_set_id": ["fallback-456"]
            }
        }]
    });

    assert_eq!(
        extract_label_set_id(&response).as_deref(),
        Some("fallback-456")
    );
}

#[test]
fn extract_inline_label_raw_mode_preserves_truncated_raw_subsections() {
    let response = serde_json::json!({
        "results": [{
            "indications_and_usage": [
                "1 INDICATIONS AND USAGE",
                "(1.1) KEYTRUDA, in combination with chemotherapy, is indicated for the treatment of patients with high-risk early-stage triple-negative breast cancer."
            ],
            "warnings_and_cautions": ["Warnings"],
            "dosage_and_administration": ["Dosage"]
        }]
    });

    let label = extract_inline_label(&response, true).expect("raw label");
    assert!(!label.indication_summary.is_empty());
    assert!(label.indications.as_deref().is_some());
    assert!(label.warnings.as_deref().is_some());
    assert!(label.dosage.as_deref().is_some());
}

#[test]
fn extract_inline_label_raw_mode_reads_boxed_warning_alongside_modern_warnings() {
    let response = serde_json::json!({
        "results": [{
            "boxed_warning": ["WARNING: SERIOUS SKIN REACTIONS"],
            "warnings_and_cautions": ["Immune-mediated adverse reactions."],
            "indications_and_usage": ["1 INDICATIONS AND USAGE", "(1.1) Indicated for melanoma."]
        }]
    });

    let label = extract_inline_label(&response, true).expect("raw label");
    assert_eq!(
        label.boxed_warning.as_deref(),
        Some("WARNING: SERIOUS SKIN REACTIONS")
    );
    assert_eq!(
        label.warnings.as_deref(),
        Some("Immune-mediated adverse reactions.")
    );
}

#[test]
fn extract_inline_label_falls_back_to_legacy_warnings_field() {
    let response = serde_json::json!({
        "results": [{
            "warnings": ["Older-format label warnings text."]
        }]
    });

    let label = extract_inline_label(&response, true).expect("raw label");
    assert_eq!(
        label.warnings.as_deref(),
        Some("Older-format label warnings text.")
    );
    assert!(label.boxed_warning.is_none());
    assert_eq!(
        extract_label_warnings_text(&response).as_deref(),
        Some("Older-format label warnings text.")
    );
}

#[test]
fn extract_label_warnings_prefers_warnings_and_cautions_over_legacy_warnings() {
    let response = serde_json::json!({
        "results": [{
            "warnings_and_cautions": ["Modern warnings text."],
            "warnings": ["Legacy warnings text."]
        }]
    });

    assert_eq!(
        extract_label_warnings_text(&response).as_deref(),
        Some("Modern warnings text.")
    );
}

#[test]
fn extract_label_warnings_truncates_with_exact_dailymed_link() {
    let response = serde_json::json!({
        "results": [{
            "set_id": "warning-set-123",
            "warnings_and_cautions": ["w".repeat(LABEL_MAX_CHARS + 7)]
        }]
    });

    let warnings = extract_label_warnings_text(&response).expect("warnings");
    assert_eq!(
        warnings.chars().take(LABEL_MAX_CHARS).count(),
        LABEL_MAX_CHARS
    );
    assert!(warnings.ends_with(
        "(truncated, 2007 chars total; full label: https://dailymed.nlm.nih.gov/dailymed/drugInfo.cfm?setid=warning-set-123)"
    ));
}

#[test]
fn extract_inline_label_summary_mode_keeps_boxed_warning() {
    let response = serde_json::json!({
        "results": [{
            "boxed_warning": ["WARNING: BOXED ONLY IN SUMMARY MODE"],
            "warnings_and_cautions": ["Immune-mediated adverse reactions."],
            "indications_and_usage": [
                "1 INDICATIONS AND USAGE",
                "(1.1) KEYTRUDA is indicated for melanoma."
            ]
        }]
    });

    let label = extract_inline_label(&response, false).expect("summary label");
    assert_eq!(
        label.boxed_warning.as_deref(),
        Some("WARNING: BOXED ONLY IN SUMMARY MODE")
    );
    assert!(label.warnings.is_none());
}

#[test]
fn extract_inline_label_boxed_only_still_returns_a_label() {
    let response = serde_json::json!({
        "results": [{
            "boxed_warning": ["WARNING: NO OTHER FIELDS"]
        }]
    });

    let label = extract_inline_label(&response, false).expect("boxed-only label");
    assert_eq!(
        label.boxed_warning.as_deref(),
        Some("WARNING: NO OTHER FIELDS")
    );
    assert!(label.warnings.is_none());
    assert!(label.indications.is_none());
    assert!(label.dosage.is_none());
}

#[test]
fn extract_label_boxed_warning_reads_all_strings_from_the_first_result() {
    let response = serde_json::json!({
        "results": [
            {"boxed_warning": ["WARNING: FIRST PART", "SECOND PART"]},
            {"boxed_warning": ["WARNING: SECOND RESULT"]}
        ]
    });

    assert_eq!(
        extract_label_boxed_warning(&response).as_deref(),
        Some("WARNING: FIRST PART SECOND PART")
    );
    assert!(extract_label_boxed_warning(&serde_json::json!({"results": []})).is_none());
}

#[test]
fn extract_label_boxed_warning_is_truncated() {
    let response = serde_json::json!({
        "results": [{"boxed_warning": ["x".repeat(LABEL_MAX_CHARS + 1)]}]
    });

    let warning = extract_label_boxed_warning(&response).expect("boxed warning");
    assert!(warning.ends_with("(truncated, 2001 chars total)"));
}
