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
    };

    assert!(search_result_quality_score(&rich) > search_result_quality_score(&sparse));
}
