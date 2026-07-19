use super::super::DiseasePhenotype;
use super::super::test_support::*;
use super::*;

struct TestEnv {
    previous: Vec<(&'static str, Option<std::ffi::OsString>)>,
}

impl TestEnv {
    fn new() -> Self {
        Self {
            previous: Vec::new(),
        }
    }

    fn set(&mut self, key: &'static str, value: &str) {
        self.previous.push((key, std::env::var_os(key)));
        // SAFETY: this test is serialized with every peer that mutates provider roots.
        unsafe { std::env::set_var(key, value) };
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        for (key, previous) in self.previous.drain(..).rev() {
            // SAFETY: this test is serialized with every peer that mutates provider roots.
            unsafe {
                if let Some(value) = previous {
                    std::env::set_var(key, value);
                } else {
                    std::env::remove_var(key);
                }
            }
        }
    }
}

fn clinical_feature_row(hpo_id: &str) -> DiseasePhenotype {
    DiseasePhenotype {
        hpo_id: hpo_id.to_string(),
        name: Some("Example phenotype".to_string()),
        evidence: Some("IEA".to_string()),
        frequency: None,
        frequency_qualifier: None,
        onset_qualifier: None,
        sex_qualifier: None,
        stage_qualifier: None,
        qualifiers: Vec::new(),
        source: Some("infores:hpo-annotations".to_string()),
    }
}

fn diagnostic_row(
    source: &str,
    accession: &str,
    name: &str,
    conditions: &[&str],
) -> crate::entities::diagnostic::DiagnosticSearchResult {
    crate::entities::diagnostic::DiagnosticSearchResult {
        source: source.to_string(),
        accession: accession.to_string(),
        name: name.to_string(),
        test_type: Some("molecular".to_string()),
        manufacturer_or_lab: Some("Example Lab".to_string()),
        genes: Vec::new(),
        conditions: conditions
            .iter()
            .map(|condition| condition.to_string())
            .collect(),
    }
}

fn seer_catalog_fixture() -> SeerSiteCatalog {
    let body = serde_json::to_vec(&serde_json::json!({
        "VariableFormats": {
            "site": {
                "1": "All Cancer Sites Combined",
                "83": "Hodgkin Lymphoma",
                "97": "Chronic Myeloid Leukemia (CML)"
            },
            "sex": {
                "1": "Both Sexes",
                "2": "Male",
                "3": "Female"
            },
            "race": {
                "1": "All Races / Ethnicities"
            },
            "age_range": {
                "1": "All Ages"
            }
        },
        "CancerSites": [
            {"value": 1, "active": true},
            {"value": 83, "active": true},
            {"value": 97, "active": true}
        ]
    }))
    .expect("catalog json");

    SeerClient::decode_site_catalog_response(
        reqwest::StatusCode::OK,
        Some(&reqwest::header::HeaderValue::from_static(
            "application/json",
        )),
        &body,
    )
    .expect("valid SEER catalog")
}

#[test]
fn funding_query_prefers_free_text_lookup() {
    let disease = test_disease(
        "MONDO:0011996",
        "chronic myelogenous leukemia, BCR-ABL1 positive",
    );

    assert_eq!(
        disease_funding_query_value(&disease, Some("chronic myeloid leukemia")),
        Some("chronic myeloid leukemia".to_string())
    );
}

#[test]
fn funding_query_uses_canonical_name_for_identifier_lookups() {
    let disease = test_disease(
        "MONDO:0011996",
        "chronic myelogenous leukemia, BCR-ABL1 positive",
    );

    assert_eq!(
        disease_funding_query_value(&disease, Some("MONDO:0011996")),
        Some("chronic myelogenous leukemia, BCR-ABL1 positive".to_string())
    );
}

#[tokio::test]
async fn apply_requested_sections_clears_funding_when_not_requested() {
    let mut disease = test_disease("MONDO:0007947", "Marfan syndrome");
    disease.funding = Some(empty_funding_section("Marfan syndrome".to_string()));
    disease.funding_note = Some(FUNDING_NO_DATA_NOTE.to_string());

    apply_requested_sections(&mut disease, DiseaseSections::default(), None)
        .await
        .expect("sections should apply");

    assert!(disease.funding.is_none());
    assert!(disease.funding_note.is_none());
}

#[tokio::test]
async fn apply_requested_sections_clears_clinical_features_when_not_requested() {
    let mut disease = test_disease("MONDO:0005105", "melanoma");
    disease
        .clinical_features
        .push(clinical_feature_row("HP:0000132"));

    apply_requested_sections(&mut disease, DiseaseSections::default(), None)
        .await
        .expect("sections should apply");

    assert!(disease.clinical_features.is_empty());
}

#[test]
fn disease_diagnostics_section_populates_from_rows() {
    let mut disease = test_disease("MONDO:0018076", "tuberculosis");
    let outcome = apply_diagnostics_section_result(
        &mut disease,
        Ok(SearchPage::offset(
            vec![
                diagnostic_row(
                    crate::entities::diagnostic::DIAGNOSTIC_SOURCE_WHO_IVD,
                    "WHO-IVD-1",
                    "Loopamp MTBC Detection Kit",
                    &["Mycobacterium tuberculosis complex (MTBC)"],
                ),
                diagnostic_row(
                    crate::entities::diagnostic::DIAGNOSTIC_SOURCE_GTR,
                    "GTR000000002.1",
                    "Tuberculosis Molecular Panel",
                    &["tuberculosis"],
                ),
            ],
            Some(12),
        )),
    );

    let rows = disease.diagnostics.as_ref().expect("diagnostics rows");
    assert_eq!(rows.len(), 2);
    assert_eq!(
        outcome.outcome(),
        crate::entities::section_outcome::SectionOutcomeState::Data
    );
    assert_eq!(
        disease.diagnostics_note.as_deref(),
        Some(
            "Showing 2 of 12 diagnostic matches in this disease card. Use diagnostic search with --limit and --offset for the larger result set."
        )
    );
    assert!(rows.iter().any(|row| {
        row.source == crate::entities::diagnostic::DIAGNOSTIC_SOURCE_WHO_IVD
            && row.name == "Loopamp MTBC Detection Kit"
            && row
                .conditions
                .iter()
                .any(|condition| condition == "Mycobacterium tuberculosis complex (MTBC)")
    }));
    assert!(rows.iter().any(|row| {
        row.source == crate::entities::diagnostic::DIAGNOSTIC_SOURCE_GTR
            && row.name.starts_with("Tuberculosis Molecular Panel")
    }));
}

#[test]
fn disease_diagnostics_unavailable_sets_note() {
    let mut disease = test_disease("MONDO:0018076", "tuberculosis");
    let outcome = apply_diagnostics_section_result(
        &mut disease,
        Err(BioMcpError::SourceUnavailable {
            source_name: "gtr".to_string(),
            reason: "fixture directory is unavailable".to_string(),
            suggestion: "Run `biomcp gtr sync`".to_string(),
        }),
    );

    assert!(disease.diagnostics.is_none());
    assert_eq!(
        outcome.outcome(),
        crate::entities::section_outcome::SectionOutcomeState::Unavailable
    );
    assert!(outcome.sources().is_empty());
    assert_eq!(
        disease.diagnostics_note.as_deref(),
        Some(DISEASE_DIAGNOSTICS_UNAVAILABLE_NOTE)
    );
}

#[test]
fn survival_catalog_resolution_sets_truthful_note_for_unmapped_disease() {
    let mut disease = test_disease("MONDO:0007947", "Marfan syndrome");

    let site = resolve_survival_site_from_catalog_result(&mut disease, Ok(seer_catalog_fixture()));

    assert!(site.is_none());
    assert!(disease.survival.is_none());
    assert_eq!(
        disease.survival_note.as_deref(),
        Some(SURVIVAL_NO_DATA_NOTE)
    );
    let outcome = disease
        .section_outcomes
        .get(DISEASE_SECTION_SURVIVAL)
        .expect("survival outcome");
    assert_eq!(
        outcome.outcome(),
        crate::entities::section_outcome::SectionOutcomeState::Empty
    );
    assert_eq!(outcome.sources(), &["SEER Explorer"]);
}

#[test]
fn survival_catalog_resolution_sets_unavailable_note_when_catalog_fails() {
    let mut disease = test_disease("MONDO:0004952", "Hodgkin's lymphoma");

    let site = resolve_survival_site_from_catalog_result(
        &mut disease,
        Err(BioMcpError::Api {
            api: "SEER Explorer".into(),
            message: "catalog failed".into(),
        }),
    );

    assert!(site.is_none());
    assert!(disease.survival.is_none());
    assert_eq!(
        disease.survival_note.as_deref(),
        Some(SURVIVAL_UNAVAILABLE_NOTE)
    );
    let outcome = disease
        .section_outcomes
        .get(DISEASE_SECTION_SURVIVAL)
        .expect("survival outcome");
    assert_eq!(
        outcome.outcome(),
        crate::entities::section_outcome::SectionOutcomeState::Unavailable
    );
    assert!(outcome.sources().is_empty());
}

#[tokio::test]
async fn ticket_589_id_only_disease_enrichments_are_inapplicable_without_credit() {
    let mut disease = test_disease("MONDO:0005105", "");
    disease.synonyms.clear();

    add_treatment_landscape(&mut disease)
        .await
        .expect("ID-only treatment lookup should not contact a provider");
    add_recruiting_trial_count(&mut disease)
        .await
        .expect("ID-only trial lookup should not contact a provider");

    for key in ["treatments", "recruiting_trials"] {
        let outcome = disease
            .section_outcomes
            .get(key)
            .unwrap_or_else(|| panic!("missing outcome-only state for {key}"));
        assert_eq!(
            outcome.outcome(),
            crate::entities::section_outcome::SectionOutcomeState::Inapplicable,
            "key={key}"
        );
        assert!(outcome.sources().is_empty(), "key={key}");
    }
    assert!(disease.treatment_landscape.is_empty());
    assert!(disease.recruiting_trial_count.is_none());
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn ticket_589_disease_base_enrichment_failures_are_unavailable_without_credit() {
    let mut env = TestEnv::new();
    env.set("BIOMCP_MYCHEM_BASE", "://invalid-mychem-fixture");
    env.set("BIOMCP_CTGOV_BASE", "://invalid-ctgov-fixture");
    env.set("BIOMCP_OPENTARGETS_BASE", "://invalid-opentargets-fixture");

    let mut treatments = test_disease("MONDO:0005105", "melanoma");
    let treatment_error = add_treatment_landscape(&mut treatments).await;
    assert!(
        treatment_error.is_err(),
        "fixture must induce a source error"
    );

    let mut trials = test_disease("MONDO:0005105", "melanoma");
    let trial_error = add_recruiting_trial_count(&mut trials).await;
    assert!(trial_error.is_err(), "fixture must induce a source error");

    for (disease, key, provider) in [
        (&treatments, "treatments", "MyChem.info indication search"),
        (&trials, "recruiting_trials", "ClinicalTrials.gov"),
    ] {
        let outcome = disease
            .section_outcomes
            .get(key)
            .unwrap_or_else(|| panic!("missing outcome-only state for {key}"));
        assert_eq!(
            outcome.outcome(),
            crate::entities::section_outcome::SectionOutcomeState::Unavailable,
            "key={key}"
        );
        assert!(outcome.sources().is_empty(), "key={key}");
        assert!(
            outcome
                .message()
                .is_some_and(|message| !message.trim().is_empty()),
            "key={key}"
        );
        assert!(
            !serde_json::to_string(outcome)
                .expect("outcome serializes")
                .contains(provider),
            "failed provider was credited: key={key}"
        );
    }
    assert!(treatments.treatment_landscape.is_empty());
    assert!(trials.recruiting_trial_count.is_none());

    let mut base_card = test_disease("MONDO:0005105", "melanoma");
    enrich_base_context(&mut base_card).await;
    let provenance = crate::render::provenance::disease_section_sources(&base_card);
    for (key, provider) in [
        ("treatments", "MyChem.info indication search"),
        ("recruiting_trials", "ClinicalTrials.gov"),
    ] {
        let section = provenance
            .iter()
            .find(|section| section.key == key)
            .unwrap_or_else(|| panic!("missing failed enrichment provenance: {key}"));
        assert_eq!(
            section.outcome,
            crate::entities::section_outcome::SectionOutcomeState::Unavailable
        );
        assert!(section.sources.is_empty());
        assert!(
            provenance
                .iter()
                .all(|section| section.sources.iter().all(|source| source != provider))
        );
    }
    let markdown = crate::render::markdown::disease_markdown(&base_card, &[])
        .expect("failed optional enrichments should still render");
    assert!(markdown.contains(TREATMENTS_UNAVAILABLE_NOTE));
    assert!(markdown.contains(RECRUITING_TRIALS_UNAVAILABLE_NOTE));
    assert!(!markdown.contains("Source: MyChem.info indication search"));
    assert!(!markdown.contains("Source: ClinicalTrials.gov"));
}

pub(crate) async fn proof_enrich_sparse_disease_identity_prefers_exact_ols4_match() {
    let mut disease = test_disease("MONDO:0019468", "MONDO:0019468");
    apply_sparse_disease_identity_docs(
        &mut disease,
        "MONDO:0019468",
        vec![
            ols_doc("MONDO:0019469", "wrong disease", &["Wrong"]),
            ols_doc(
                "MONDO:0019468",
                "T-cell prolymphocytic leukemia",
                &["T-PLL"],
            ),
        ],
    );

    assert_eq!(disease.name, "T-cell prolymphocytic leukemia");
    assert_eq!(disease.synonyms, vec!["T-PLL".to_string()]);
}

#[tokio::test]
async fn enrich_sparse_disease_identity_prefers_exact_ols4_match() {
    proof_enrich_sparse_disease_identity_prefers_exact_ols4_match().await;
}

fn ols_doc(id: &str, label: &str, synonyms: &[&str]) -> crate::sources::ols4::OlsDoc {
    crate::sources::ols4::OlsDoc {
        iri: format!("http://purl.obolibrary.org/obo/{}", id.replace(':', "_")),
        ontology_name: "mondo".into(),
        ontology_prefix: "mondo".into(),
        short_form: Some(id.replace(':', "_")),
        obo_id: Some(id.into()),
        label: label.into(),
        description: Vec::new(),
        exact_synonyms: synonyms.iter().map(|value| (*value).to_string()).collect(),
        is_defining_ontology: false,
        doc_type: Some("class".into()),
    }
}
