//! Sidecar tests for variant MyVariant search helpers.

use super::super::{VariantProteinAlias, VariantSearchFilters, VariantSearchResult};
use super::*;

#[test]
fn search_query_summary_includes_hgvsc_and_rsid() {
    let summary = search_query_summary(&VariantSearchFilters {
        gene: Some("BRAF".into()),
        hgvsc: Some("c.1799T>A".into()),
        rsid: Some("rs113488022".into()),
        ..Default::default()
    });
    assert_eq!(summary, "gene=BRAF, hgvsc=c.1799T>A, rsid=rs113488022");
}

#[test]
fn search_query_summary_includes_residue_alias_marker() {
    let summary = search_query_summary(&VariantSearchFilters {
        gene: Some("PTPN22".into()),
        protein_alias: Some(VariantProteinAlias {
            position: 620,
            residue: 'W',
        }),
        ..Default::default()
    });
    assert_eq!(summary, "gene=PTPN22, residue_alias=620W");
}

#[test]
fn quality_score_prioritizes_significance_and_frequency() {
    let rich = VariantSearchResult {
        id: "chr1:g.1A>T".into(),
        gene: "TP53".into(),
        hgvs_p: Some("p.V1A".into()),
        legacy_name: None,
        significance: Some("Pathogenic".into()),
        clinvar_stars: None,
        gnomad_af: Some(0.001),
        revel: None,
        gerp: None,
        source_identity: None,
        matched_alias: None,
    };
    let sparse = VariantSearchResult {
        id: "chr1:g.2A>T".into(),
        gene: "TP53".into(),
        hgvs_p: Some("p.V2A".into()),
        legacy_name: None,
        significance: None,
        clinvar_stars: None,
        gnomad_af: None,
        revel: None,
        gerp: None,
        source_identity: None,
        matched_alias: None,
    };

    assert!(search_result_quality_score(&rich) > search_result_quality_score(&sparse));
}

#[test]
fn resolution_status_follows_compatible_indeterminate_and_exhaustive_truth_table() {
    assert_eq!(
        resolution_status(1, false, true),
        VariantResolutionStatus::Resolved
    );
    assert_eq!(
        resolution_status(2, false, true),
        VariantResolutionStatus::Ambiguous
    );
    assert_eq!(
        resolution_status(0, true, true),
        VariantResolutionStatus::Ambiguous
    );
    assert_eq!(
        resolution_status(1, false, false),
        VariantResolutionStatus::Ambiguous
    );
    assert_eq!(
        resolution_status(0, false, true),
        VariantResolutionStatus::Unresolved
    );
}

#[test]
fn candidate_scan_exhaustion_requires_empty_page_or_reaching_provider_total() {
    assert!(!candidate_scan_exhaustive(Some(10), 5, 5));
    assert!(!candidate_scan_exhaustive(None, 5, 5));
    assert!(candidate_scan_exhaustive(Some(5), 5, 5));
    assert!(candidate_scan_exhaustive(Some(10), 5, 0));
}

#[test]
fn normalized_source_identity_key_deduplicates_alias_order_and_spelling() {
    let first = SourceVariantIdentity {
        genomic_id: "chr7:g.1A>T".into(),
        genes: vec!["BRAF".into()],
        protein_changes: vec!["p.Val600Glu".into(), "p.V600E".into()],
        coding_changes: vec!["NM_004333.6:c.1799T>A".into()],
        rsids: Vec::new(),
    };
    let second = SourceVariantIdentity {
        protein_changes: vec!["V600E".into()],
        coding_changes: vec!["c.1799T>A".into()],
        ..first.clone()
    };
    assert_eq!(first.normalized_key(), second.normalized_key());
}

fn hit(id: &str, protein_change: Option<&str>) -> MyVariantHit {
    let dbnsfp =
        protein_change.map(|change| serde_json::json!({"genename": "BRAF", "hgvsp": change}));
    serde_json::from_value(serde_json::json!({"_id": id, "dbnsfp": dbnsfp}))
        .expect("valid MyVariant hit")
}

#[test]
fn exact_aggregation_filters_before_pagination_and_keeps_cap_truthful() {
    let requested = RequestedVariantIdentity::for_search(
        Some("BRAF".into()),
        Some("p.Val600Glu".into()),
        None,
        None,
    );
    let mut seen = HashSet::new();
    let mut retained = Vec::new();

    assert!(!retain_compatible_hits(
        &requested,
        [hit("chr7:g.1A>T", Some("p.V601E"))],
        &mut seen,
        &mut retained,
    ));
    assert!(retained.is_empty());
    assert!(!retain_compatible_hits(
        &requested,
        [
            hit("chr7:g.2A>T", Some("p.V600E")),
            hit("chr7:g.2A>T", Some("p.Val600Glu")),
        ],
        &mut seen,
        &mut retained,
    ));

    let page = finalize_exact_page(&requested, retained.clone(), 0, 1, false, true);
    assert_eq!(page.results[0].id, "chr7:g.2A>T");
    assert_eq!(page.total, Some(1));
    assert_eq!(page.has_more, Some(false));
    assert_eq!(
        page.resolution.expect("exact resolution").status,
        VariantResolutionStatus::Resolved
    );

    let capped = finalize_exact_page(&requested, retained, 0, 1, false, false);
    assert_eq!(capped.total, None);
    assert_eq!(capped.has_more, Some(true));
    assert_eq!(
        capped.resolution.expect("exact resolution").status,
        VariantResolutionStatus::Ambiguous
    );
}

#[test]
fn exact_aggregation_marks_missing_identity_evidence_indeterminate() {
    let requested =
        RequestedVariantIdentity::for_search(Some("BRAF".into()), Some("V600E".into()), None, None);
    let mut seen = HashSet::new();
    let mut retained = Vec::new();
    assert!(retain_compatible_hits(
        &requested,
        [hit("chr7:g.1A>T", None)],
        &mut seen,
        &mut retained,
    ));
    assert!(retained.is_empty());
}
