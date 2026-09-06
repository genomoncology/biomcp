//! Tier 3 - local-data parsing and lookup. Pure: parses CVX/MVX fixture text and
//! checks alias lookup behavior. No network.

use super::super::*;
use crate::test_support::TempDirGuard;
use reqwest::header::HeaderValue;

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
fn parse_cvx_codes_parses_real_shape_and_non_vaccine_flag() {
    let root = TempDirGuard::new("cvx-parse");
    let path = root.path().join("cvx.txt");
    std::fs::write(
        &path,
        "62|HPV, quadrivalent|human papilloma virus vaccine, quadrivalent||Active|False|2020/06/02\n27|botulinum antitoxin|botulinum antitoxin||Active|True|2020/09/04\n",
    )
    .expect("write fixture");

    let rows = parse_cvx_codes(
        &path,
        &std::fs::read_to_string(&path).expect("read fixture"),
    )
    .expect("parse cvx codes");

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].cvx_code, "62");
    assert_eq!(rows[0].short_description, "HPV, quadrivalent");
    assert!(!rows[0].non_vaccine);
    assert!(rows[1].non_vaccine);
}

#[test]
fn parse_cvx_products_handles_trailing_blank_field() {
    let root = TempDirGuard::new("cvx-products");
    let path = root.path().join(TRADENAME_FILE);
    std::fs::write(
        &path,
        "PREVNAR 13|Pneumococcal conjugate PCV 13|133|Pfizer, Inc|PFR|Active|Active|2010/05/28|\n",
    )
    .expect("write fixture");

    let rows = parse_cvx_products(
        &path,
        &std::fs::read_to_string(&path).expect("read fixture"),
    )
    .expect("parse tradename file");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].product_name, "PREVNAR 13");
    assert_eq!(rows[0].cvx_code, "133");
}

#[test]
fn parse_mvx_rows_rejects_short_rows() {
    let root = TempDirGuard::new("cvx-bad-mvx");
    let path = root.path().join("mvx.txt");
    std::fs::write(&path, "PFR|Pfizer, Inc|broken\n").expect("write fixture");

    let err = parse_mvx_rows(
        &path,
        &std::fs::read_to_string(&path).expect("read fixture"),
    )
    .expect_err("short mvx row should error");

    assert!(format!("{err:?}").contains("expected at least 5 fields"));
}

#[test]
fn lookup_brand_aliases_supports_exact_and_family_prefix_matching() {
    let root = TempDirGuard::new("cvx-lookup");
    super::write_fixture_bundle(root.path());
    let client = CvxClient::from_root(root.path().to_path_buf());

    assert_eq!(
        client
            .lookup_brand_aliases("prevnar")
            .expect("prevnar lookup"),
        vec![
            "Pneumococcal conjugate PCV 13".to_string(),
            "pneumococcal conjugate vaccine, 13 valent".to_string(),
        ]
    );
    assert_eq!(
        client
            .lookup_brand_aliases("fluzone")
            .expect("fluzone lookup"),
        vec![
            "Influenza, split virus, trivalent, PF".to_string(),
            "Influenza, split virus, trivalent, injectable, preservative free".to_string(),
            "Influenza, split virus, trivalent, preservative".to_string(),
            "Influenza, split virus, trivalent, injectable, contains preservative".to_string(),
        ]
    );
}

#[test]
fn lookup_brand_aliases_prefers_exact_product_before_family_prefix_and_dedupes() {
    let root = TempDirGuard::new("cvx-ranking");
    super::write_fixture_bundle(root.path());
    let client = CvxClient::from_root(root.path().to_path_buf());

    assert_eq!(
        client
            .lookup_brand_aliases("gardasil")
            .expect("gardasil lookup"),
        vec![
            "HPV, quadrivalent".to_string(),
            "human papilloma virus vaccine, quadrivalent".to_string(),
            "HPV9".to_string(),
            "Human Papillomavirus 9-valent vaccine".to_string(),
        ]
    );
}

#[test]
fn lookup_brand_aliases_matches_cvx_family_terms_for_antigen_queries() {
    let root = TempDirGuard::new("cvx-antigen");
    super::write_fixture_bundle(root.path());
    let client = CvxClient::from_root(root.path().to_path_buf());

    assert_eq!(
        client.lookup_brand_aliases("HPV").expect("HPV lookup"),
        vec![
            "HPV9".to_string(),
            "Human Papillomavirus 9-valent vaccine".to_string(),
            "HPV, quadrivalent".to_string(),
            "human papilloma virus vaccine, quadrivalent".to_string(),
        ]
    );
}

#[test]
fn ema_search_aliases_require_trade_name_identity() {
    let root = TempDirGuard::new("cvx-ema-trade-name-only");
    super::write_fixture_bundle(root.path());
    let client = CvxClient::from_root(root.path().to_path_buf());

    assert!(
        client
            .lookup_brand_search_aliases("HPV")
            .expect("strict EMA aliases")
            .is_empty()
    );
    let gardasil = client
        .lookup_brand_search_aliases("gardasil")
        .expect("Gardasil aliases");
    assert_eq!(gardasil[0].value, "HPV, quadrivalent");
    assert_eq!(gardasil[0].source, CvxAliasSource::ShortDescription);
    assert_eq!(gardasil[1].source, CvxAliasSource::FullVaccineName);
}

#[test]
fn lookup_brand_aliases_joins_mvx_rows_when_present() {
    let root = TempDirGuard::new("cvx-mvx");
    super::write_fixture_bundle(root.path());
    let client = CvxClient::from_root(root.path().to_path_buf());

    let records = client.read_alias_records().expect("read alias records");
    let prevnar = records
        .iter()
        .find(|record| record.product_name == "PREVNAR 13")
        .expect("prevnar record");
    assert_eq!(prevnar.mvx_code.as_deref(), Some("PFR"));
    assert_eq!(
        prevnar.mvx_manufacturer_name.as_deref(),
        Some("Pfizer, Inc")
    );
}

#[test]
fn lookup_brand_aliases_skips_non_vaccine_rows() {
    let root = TempDirGuard::new("cvx-non-vaccine");
    super::write_fixture_bundle(root.path());
    let client = CvxClient::from_root(root.path().to_path_buf());

    assert!(
        client
            .lookup_brand_aliases("nevermatch")
            .expect("lookup should succeed")
            .is_empty()
    );
}

#[test]
fn lookup_vaccine_candidates_returns_cvx_codes_for_brand_matches() {
    let root = TempDirGuard::new("cvx-candidates");
    super::write_fixture_bundle(root.path());
    let client = CvxClient::from_root(root.path().to_path_buf());

    let candidates = client
        .lookup_vaccine_candidates("comirnaty")
        .expect("candidate lookup");

    assert_eq!(
        candidates
            .iter()
            .map(|candidate| candidate.cvx_code.as_str())
            .collect::<Vec<_>>(),
        vec!["208", "217"]
    );
}
