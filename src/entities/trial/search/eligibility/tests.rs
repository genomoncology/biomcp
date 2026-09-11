//! Tests for trial eligibility and facility-geo helpers.

use super::super::super::test_support::*;
use super::*;

fn receipted_ctgov_detail() -> biodata::ClinicalTrialsGovApiV2Response {
    ClinicalTrialsClient::decode_biodata_detail_response(
        "NCT02576665",
        &["locations".to_owned()],
        reqwest::StatusCode::OK,
        include_bytes!("../../../../../testdata/sources/ctgov/get_nct02576665_full_20260903.json"),
    )
    .expect("receipted CTGov detail")
}

#[test]
fn split_eligibility_sections_detects_exclusion_header() {
    let text =
        "Inclusion Criteria:\nMust have MSI-H disease\n\nExclusion Criteria:\nNo active CNS mets";
    let (inclusion, exclusion) = split_eligibility_sections(text);
    assert!(inclusion.contains("must have msi-h disease"));
    assert!(exclusion.contains("no active cns mets"));
}

#[test]
fn split_eligibility_sections_supports_key_exclusion_header() {
    let text =
        "Inclusion:\nBRAF V600E mutation\n\nKey Exclusion Criteria:\nPrior anti-braf therapy";
    let (inclusion, exclusion) = split_eligibility_sections(text);
    assert!(inclusion.contains("braf v600e mutation"));
    assert!(exclusion.contains("prior anti-braf therapy"));
}

#[test]
fn split_eligibility_sections_without_exclusion_keeps_all_in_inclusion() {
    let text = "Inclusion Criteria:\nPathogenic EGFR mutation";
    let (inclusion, exclusion) = split_eligibility_sections(text);
    assert!(inclusion.contains("pathogenic egfr mutation"));
    assert!(exclusion.is_empty());
}

#[test]
fn eligibility_keyword_in_inclusion_keeps_when_inclusion_matches() {
    assert!(eligibility_keyword_in_inclusion(
        "must have msi-h disease",
        "no untreated brain metastases",
        "MSI-H"
    ));
}

#[test]
fn eligibility_keyword_in_inclusion_discards_exclusion_only_match() {
    assert!(!eligibility_keyword_in_inclusion(
        "must have metastatic colorectal cancer",
        "exclusion includes msi-h tumors",
        "MSI-H"
    ));
}

#[test]
fn eligibility_keyword_in_inclusion_keeps_when_in_both_sections() {
    assert!(eligibility_keyword_in_inclusion(
        "inclusion requires braf v600e mutation",
        "exclude prior braf v600e inhibitor exposure",
        "BRAF V600E"
    ));
}

#[test]
fn eligibility_keyword_in_inclusion_discards_negated_inclusion_sentence() {
    assert!(!eligibility_keyword_in_inclusion(
        "patients whose tumors are msi-h are excluded",
        "exclude active infection",
        "MSI-H"
    ));
}

#[test]
fn eligibility_keyword_in_inclusion_fails_open_when_keyword_missing() {
    assert!(eligibility_keyword_in_inclusion(
        "include untreated metastatic disease",
        "exclude uncontrolled infection",
        "MSI-H"
    ));
}

#[test]
fn eligibility_keyword_in_inclusion_fails_open_without_exclusion_section() {
    assert!(eligibility_keyword_in_inclusion(
        "patients with msi-h disease",
        "",
        "MSI-H"
    ));
}

#[test]
fn eligibility_keyword_in_inclusion_rejects_negated_without_exclusion_section() {
    assert!(!eligibility_keyword_in_inclusion(
        "participants must not have previously received osimertinib",
        "",
        "osimertinib"
    ));
}

#[test]
fn eligibility_keyword_in_inclusion_rejects_no_prior_without_exclusion_section() {
    assert!(!eligibility_keyword_in_inclusion(
        "no prior osimertinib therapy allowed",
        "",
        "osimertinib"
    ));
}

#[test]
fn eligibility_keyword_in_inclusion_rejects_mixed_context_without_exclusion_section() {
    assert!(!eligibility_keyword_in_inclusion(
        "participants must not have previously received osimertinib. \
         inability to swallow osimertinib tablets. \
         duration before restarting osimertinib is advised",
        "",
        "osimertinib"
    ));
}

#[test]
fn verify_age_eligibility_handles_sub_year_minimum_age() {
    let study = ctgov_search_results(vec![ctgov_search_study_fixture(
        "NCT00000001",
        "6 Months",
        "75 Years",
    )])
    .remove(0);

    assert!(verify_age_eligibility(vec![study.clone()], 0.0).is_empty());
    assert_eq!(verify_age_eligibility(vec![study], 0.5).len(), 1);
}

#[test]
fn verify_age_eligibility_handles_sub_year_maximum_age() {
    let study = ctgov_search_results(vec![ctgov_search_study_fixture(
        "NCT00000002",
        "0 Years",
        "6 Months",
    )])
    .remove(0);

    assert_eq!(verify_age_eligibility(vec![study.clone()], 0.5).len(), 1);
    assert!(verify_age_eligibility(vec![study], 1.0).is_empty());
}

#[test]
fn verify_age_eligibility_honors_shared_source_stated_no_limit_maximum() {
    let study = ctgov_search_results(vec![ctgov_search_study_fixture(
        "NCT99999999",
        "18 Years",
        "999 Years",
    )])
    .remove(0);
    assert_eq!(verify_age_eligibility(vec![study], 150.0).len(), 1);
}

#[test]
fn collect_eligibility_keywords_includes_supported_filters() {
    let filters = TrialSearchFilters {
        mutation: Some("MSI-H".into()),
        criteria: Some("mismatch repair deficient".into()),
        biomarker: Some("TMB-high".into()),
        prior_therapies: Some("osimertinib".into()),
        progression_on: Some("pembrolizumab".into()),
        ..Default::default()
    };

    assert_eq!(
        collect_eligibility_keywords(&filters),
        vec![
            "MSI-H",
            "mismatch repair deficient",
            "osimertinib",
            "pembrolizumab"
        ]
    );
}

#[test]
fn collect_eligibility_keywords_omits_blank_values() {
    let filters = TrialSearchFilters {
        mutation: Some("   ".into()),
        criteria: Some("".into()),
        biomarker: Some(" MSI-H ".into()),
        prior_therapies: None,
        progression_on: Some("".into()),
        ..Default::default()
    };

    assert_eq!(collect_eligibility_keywords(&filters), Vec::<String>::new());
}

#[test]
fn collect_eligibility_keywords_skips_boolean_expressions() {
    let filters = TrialSearchFilters {
        mutation: Some("dMMR OR MSI-H".into()),
        criteria: Some("prior platinum AND ECOG 0-1".into()),
        prior_therapies: Some("pembrolizumab".into()),
        ..Default::default()
    };

    assert_eq!(
        collect_eligibility_keywords(&filters),
        vec!["pembrolizumab"]
    );
}

#[test]
fn contains_keyword_tokens_matches_plus_suffix_token() {
    assert!(contains_keyword_tokens(
        "HER2+ positive breast cancer",
        "HER2+"
    ));
}

#[test]
fn contains_keyword_tokens_does_not_match_without_plus_suffix() {
    assert!(!contains_keyword_tokens("her2 amplification", "HER2+"));
}

#[test]
fn contains_keyword_tokens_matches_slash_separated_plus_tokens() {
    assert!(contains_keyword_tokens("ER+/PR+ breast cancer", "ER+"));
}

#[test]
fn contains_keyword_tokens_matches_hyphenated_token() {
    assert!(contains_keyword_tokens("PD-L1 expression >=1%", "PD-L1"));
}

#[test]
fn contains_keyword_tokens_matches_hyphenated_plus_token() {
    assert!(contains_keyword_tokens("PD-L1+ expression >=1%", "PD-L1+"));
}

#[test]
fn contains_keyword_tokens_rejects_hyphenated_token_without_plus_suffix() {
    assert!(!contains_keyword_tokens("PD-L1 positive", "PD-L1+"));
}

#[test]
fn contains_keyword_tokens_matches_word_token() {
    assert!(contains_keyword_tokens("BRAF V600E mutation", "BRAF"));
}

#[test]
fn contains_keyword_tokens_rejects_substring_word_match() {
    assert!(!contains_keyword_tokens("abraf", "BRAF"));
}

#[test]
fn facility_geo_requires_name_and_distance_on_the_same_shared_site() {
    let study = receipted_ctgov_detail();

    assert!(!trial_matches_facility_geo(
        &study,
        "sarah cannon",
        29.76328,
        -95.36327,
        10
    ));
}

#[test]
fn facility_geo_keeps_shared_site_name_and_distance_match() {
    let study = receipted_ctgov_detail();

    assert!(trial_matches_facility_geo(
        &study,
        "sarah cannon",
        39.73915,
        -104.9847,
        10
    ));
}
