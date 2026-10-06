//! Tier 3 - CSV parsing and local result shaping. Pure: reads committed WHO
//! Prequalification fixtures and validates output rows. No network.

use super::super::*;
use crate::entities::drug::{WhoPrequalificationEntry, WhoPrequalificationKind};
use crate::test_support::TempDirGuard;
use reqwest::header::HeaderValue;

#[test]
fn boundary_phrase_rejection_advances_across_multibyte_character() {
    assert!(!contains_boundary_phrase("β-blockers", "β-blocker"));
}

#[test]
fn parsers_require_expected_headers() {
    let err =
        parse_who_pq_csv(WHO_PQ_CSV_FILE, "wrong,header\n1,2\n").expect_err("parse should fail");
    let message = format!("{err}");
    assert!(message.contains("headers did not match"));
    assert!(message.contains(WHO_PQ_CSV_FILE));
    assert!(message.contains("missing required column"));

    let err = parse_who_api_csv(WHO_PQ_API_CSV_FILE, "wrong,header\n1,2\n")
        .expect_err("parse should fail");
    let message = format!("{err}");
    assert!(message.contains("headers did not match"));
    assert!(message.contains(WHO_PQ_API_CSV_FILE));

    let err = parse_who_vaccines_csv(WHO_VACCINES_CSV_FILE, "wrong,header\n1,2\n")
        .expect_err("parse should fail");
    let message = format!("{err}");
    assert!(message.contains("headers did not match"));
    assert!(message.contains(WHO_VACCINES_CSV_FILE));
}

#[test]
fn vaccines_capture_keeps_the_trailing_space_header_and_trims_to_match() {
    // The recorded vaccines export ships "Date of Prequalification " with a
    // trailing space inside the quotes (issue #288, verified with cat -A).
    // Pin the raw byte on the recorded capture and the trim-then-compare
    // behavior that resolves it to the required column.
    let payload = super::fixture_vaccine_csv();
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(payload.as_bytes());
    let headers = reader.headers().expect("vaccine headers should read");
    assert_eq!(headers.get(0), Some("Date of Prequalification "));
    assert_eq!(
        normalize_header(headers.get(0).unwrap_or_default()),
        "DATE OF PREQUALIFICATION"
    );

    let rows = parse_who_vaccines_csv(WHO_VACCINES_CSV_FILE, &payload)
        .expect("recorded vaccines capture should parse");
    assert!(rows.iter().any(|row| row.prequalification_date.is_some()));
}

#[test]
fn medicines_captures_ship_title_case_headers_and_parse() {
    // Both medicines exports moved to title-case headers and the
    // finished-pharma export dropped "Basis of Alternative Listing"
    // (issue #288); the recorded captures must parse against the re-derived
    // header sets.
    let finished = super::fixture_csv();
    assert!(finished.starts_with("\"WHO Reference Number\""));
    assert!(!finished.contains("Basis of Alternative Listing"));
    let rows = parse_who_pq_csv(WHO_PQ_CSV_FILE, &finished)
        .expect("recorded finished-pharma capture should parse");
    assert!(
        rows.iter()
            .any(|row| row.who_reference_number.as_deref() == Some("MA051"))
    );

    let api = super::fixture_api_csv();
    assert!(api.contains(",Applicant,"));
    let rows =
        parse_who_api_csv(WHO_PQ_API_CSV_FILE, &api).expect("recorded API capture should parse");
    assert!(rows.iter().any(|row| row.who_product_id.is_some()));
}

#[test]
fn ensure_csv_content_type_rejects_html_response_without_raw_tags() {
    let content_type = HeaderValue::from_static("text/html; charset=utf-8");
    let err = ensure_csv_content_type(Some(&content_type), b"<html><body>not csv</body></html>")
        .expect_err("html should be rejected");
    let message = format!("{err:?}");

    assert!(message.contains("Unexpected HTML response"));
    assert!(message.contains("HTML error page"));
    assert!(!message.contains("<html>"));
}

#[test]
fn normalize_dates_convert_to_iso() {
    assert_eq!(
        normalize_who_date("18  Dec,  2019").as_deref(),
        Some("2019-12-18")
    );
    assert_eq!(normalize_who_date(""), None);

    assert_eq!(
        normalize_vaccine_date("09/10/2024").as_deref(),
        Some("2024-10-09")
    );
    assert_eq!(normalize_vaccine_date(""), None);
    assert_eq!(normalize_vaccine_date("00/10/2024"), None);
    assert_eq!(normalize_vaccine_date("09/13/2024"), None);
}

#[test]
fn derive_inn_removes_dosage_form_suffix_when_present() {
    assert_eq!(
        derive_inn(
            "Trastuzumab Powder for concentrate for solution for infusion 150 mg",
            "Powder for concentrate for solution for infusion"
        ),
        "Trastuzumab"
    );
}

#[test]
fn row_matching_strips_salt_suffixes_from_match_key() {
    let row = WhoPrequalificationEntry {
        kind: WhoPrequalificationKind::FinishedPharma,
        who_reference_number: Some("ANDA 077844 USFDA".to_string()),
        inn: "Abacavir (sulfate)".to_string(),
        presentation: Some("Abacavir (sulfate) Tablet 300mg".to_string()),
        dosage_form: Some("Tablet".to_string()),
        product_type: "Finished Pharmaceutical Product".to_string(),
        therapeutic_area: "HIV/AIDS".to_string(),
        applicant: "Aurobindo Pharma Ltd".to_string(),
        listing_basis: Some("Alternative Listing".to_string()),
        alternative_listing_basis: Some("USFDA - PEPFAR".to_string()),
        prequalification_date: None,
        who_product_id: None,
        grade: None,
        confirmation_document_date: None,
        vaccine_type: None,
        commercial_name: None,
        dose_count: None,
        manufacturer: None,
        responsible_nra: None,
    };

    assert!(row_matches_identity(&row, &WhoPqIdentity::new("abacavir")));
}

#[test]
fn row_matching_falls_back_to_full_presentation_for_combo_rows() {
    let rows =
        parse_who_pq_csv(WHO_PQ_CSV_FILE, &super::fixture_csv()).expect("fixture should parse");
    let combo = rows
        .into_iter()
        .find(|row| row.who_reference_number.as_deref() == Some("BT-ON017"))
        .expect("combo row should exist");

    assert!(row_matches_identity(
        &combo,
        &WhoPqIdentity::new("trastuzumab")
    ));
}

#[test]
fn parse_who_pq_csv_deduplicates_by_reference_number() {
    let payload = format!(
        "{csv}\n\"BT-ON001\",\"Trastuzumab Powder for concentrate for solution for infusion 150 mg\",\"Biotherapeutic Product\",\"Oncology\",\"Samsung Bioepis NL B.V.\",\"Powder for concentrate for solution for infusion\",\"Prequalification - Abridged\",,\"18  Dec,  2019\"\n",
        csv = super::fixture_csv().trim_end()
    );
    let rows = parse_who_pq_csv(WHO_PQ_CSV_FILE, &payload).expect("duplicate payload should parse");
    let count = rows
        .iter()
        .filter(|row| row.who_reference_number.as_deref() == Some("BT-ON001"))
        .count();
    assert_eq!(count, 1);
}

#[test]
fn parse_who_api_csv_preserves_identifier_semantics() {
    let rows = parse_who_api_csv(WHO_PQ_API_CSV_FILE, &super::fixture_api_csv())
        .expect("API fixture should parse");
    let row = rows
        .into_iter()
        .find(|row| row.who_product_id.as_deref() == Some("WHOAPI-010"))
        .expect("abacavir API row should exist");

    assert_eq!(row.who_reference_number, None);
    assert_eq!(row.who_product_id.as_deref(), Some("WHOAPI-010"));
    assert_eq!(row.presentation, None);
    assert_eq!(row.dosage_form, None);
    assert_eq!(row.listing_basis, None);
    assert_eq!(row.grade.as_deref(), Some("Standard"));
    assert_eq!(
        row.confirmation_document_date.as_deref(),
        Some("2025-09-19")
    );
}

#[test]
fn read_rows_combines_finished_pharma_api_and_vaccine_rows() {
    let root = TempDirGuard::new("who-read-rows");
    std::fs::write(root.path().join(WHO_PQ_CSV_FILE), super::fixture_csv()).expect("write WHO CSV");
    std::fs::write(
        root.path().join(WHO_PQ_API_CSV_FILE),
        super::fixture_api_csv(),
    )
    .expect("write WHO API CSV");
    std::fs::write(
        root.path().join(WHO_VACCINES_CSV_FILE),
        super::fixture_vaccine_csv(),
    )
    .expect("write WHO vaccine CSV");

    let rows = WhoPqClient::from_root(root.path())
        .read_rows()
        .expect("WHO rows should read");

    assert!(
        rows.iter()
            .any(|row| row.who_reference_number.as_deref() == Some("MA051"))
    );
    assert!(
        rows.iter()
            .any(|row| row.who_product_id.as_deref() == Some("WHOAPI-081"))
    );
    assert!(rows.iter().any(|row| {
        row.commercial_name.as_deref() == Some("Gardasil 9")
            && matches!(row.kind, WhoPrequalificationKind::Vaccine)
    }));
}

#[test]
fn product_type_filters_keep_expected_rows() {
    let rows = vec![
        parse_who_pq_csv(WHO_PQ_CSV_FILE, &super::fixture_csv())
            .expect("fixture should parse")
            .into_iter()
            .find(|row| row.who_reference_number.as_deref() == Some("MA051"))
            .expect("finished row should exist"),
        parse_who_api_csv(WHO_PQ_API_CSV_FILE, &super::fixture_api_csv())
            .expect("API fixture should parse")
            .into_iter()
            .find(|row| row.who_product_id.as_deref() == Some("WHOAPI-081"))
            .expect("API row should exist"),
        parse_who_vaccines_csv(WHO_VACCINES_CSV_FILE, &super::fixture_vaccine_csv())
            .expect("vaccine fixture should parse")
            .into_iter()
            .find(|row| row.commercial_name.as_deref() == Some("Comirnaty®"))
            .expect("vaccine row should exist"),
    ];

    let filtered = filter_rows_by_product_type(&rows, WhoProductTypeFilter::Api);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].who_product_id.as_deref(), Some("WHOAPI-081"));

    let filtered = filter_rows_by_product_type(&rows, WhoProductTypeFilter::FinishedPharma);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].who_reference_number.as_deref(), Some("MA051"));

    let filtered = filter_rows_by_product_type(&rows, WhoProductTypeFilter::Vaccine);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].commercial_name.as_deref(), Some("Comirnaty®"));
}

#[test]
fn parse_who_vaccines_csv_preserves_blank_dose_rows() {
    let rows = parse_who_vaccines_csv(WHO_VACCINES_CSV_FILE, &super::fixture_vaccine_csv())
        .expect("fixture should parse");
    let row = rows
        .into_iter()
        .find(|row| row.commercial_name.as_deref() == Some("Comirnaty®"))
        .expect("blank-dose vaccine row should exist");

    assert!(matches!(row.kind, WhoPrequalificationKind::Vaccine));
    assert_eq!(row.vaccine_type.as_deref(), Some("Covid-19"));
    assert_eq!(row.inn, "Covid-19");
    assert_eq!(row.applicant, "BioNTech Manufacturing GmbH");
    assert_eq!(row.dose_count, None);
    assert_eq!(row.prequalification_date.as_deref(), Some("2024-10-09"));
}

#[test]
fn vaccine_row_matching_uses_vaccine_type_and_brand_aliases() {
    let rows = parse_who_vaccines_csv(WHO_VACCINES_CSV_FILE, &super::fixture_vaccine_csv())
        .expect("fixture should parse");
    let bcg = rows
        .iter()
        .find(|row| row.commercial_name.as_deref() == Some("BCG Freeze Dried Glutamate vaccine"))
        .expect("BCG row should exist");
    let gardasil = rows
        .iter()
        .find(|row| row.commercial_name.as_deref() == Some("Gardasil 9"))
        .expect("Gardasil row should exist");

    assert!(row_matches_identity(bcg, &WhoPqIdentity::new("BCG")));
    assert!(row_matches_identity(
        gardasil,
        &WhoPqIdentity::new("Gardasil")
    ));
}

#[test]
fn vaccine_dedupe_keeps_distinct_bevac_rows() {
    let rows = parse_who_vaccines_csv(WHO_VACCINES_CSV_FILE, &super::fixture_vaccine_csv())
        .expect("fixture should parse");
    let bevac = rows
        .into_iter()
        .filter(|row| row.commercial_name.as_deref() == Some("BEVAC®"))
        .collect::<Vec<_>>();

    // The 2026-10-06 capture carries three BEVAC\u{ae} rows across two
    // products and three presentations; none of them may collapse.
    assert_eq!(bevac.len(), 3);
    let distinct = bevac
        .iter()
        .map(|row| row.stable_identifier_key())
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(distinct.len(), bevac.len());
}

#[test]
fn vaccine_fixture_carries_full_validation_anchor_counts() {
    let rows = parse_who_vaccines_csv(WHO_VACCINES_CSV_FILE, &super::fixture_vaccine_csv())
        .expect("fixture should parse");

    let bcg = rows
        .iter()
        .filter(|row| row.vaccine_type.as_deref() == Some("BCG"))
        .count();
    let hpv = rows
        .iter()
        .filter(|row| {
            row.vaccine_type
                .as_deref()
                .is_some_and(|value| value.contains("Human Papillomavirus"))
        })
        .count();
    let covid = rows
        .iter()
        .filter(|row| row.vaccine_type.as_deref() == Some("Covid-19"))
        .count();
    let measles = rows
        .iter()
        .filter(|row| {
            row.vaccine_type
                .as_deref()
                .is_some_and(|value| value.to_ascii_lowercase().contains("measles"))
        })
        .count();
    let yellow_fever = rows
        .iter()
        .filter(|row| row.vaccine_type.as_deref() == Some("Yellow Fever"))
        .count();

    assert_eq!(bcg, 7);
    assert_eq!(hpv, 9);
    assert_eq!(covid, 3);
    assert_eq!(measles, 22);
    assert_eq!(yellow_fever, 10);
}
