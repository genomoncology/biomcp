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
        genome_build: super::super::GenomeBuild::Grch37,
        genome_build_provenance: "test".into(),
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
        genome_build: super::super::GenomeBuild::Grch37,
        genome_build_provenance: "test".into(),
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
        genome_build: "GRCh37".into(),
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
fn exact_aggregation_retains_identical_complex_protein_hgvs() {
    let requested = RequestedVariantIdentity::for_search(
        Some("EGFR".into()),
        Some("p.Glu746_Ala750del".into()),
        None,
        None,
    );
    let source_hit = serde_json::from_value(serde_json::json!({
        "_id": "chr7:g.55242465_55242479del",
        "dbnsfp": {
            "genename": "EGFR",
            "hgvsp": "NP_005219.2:p.Glu746_Ala750del"
        }
    }))
    .expect("valid MyVariant hit");
    let mut seen = HashSet::new();
    let mut retained = Vec::new();

    assert!(!retain_compatible_hits(
        &requested,
        [source_hit],
        &mut seen,
        &mut retained,
    ));
    assert_eq!(
        retained[0].matched_alias.as_deref(),
        Some("NP_005219.2:p.Glu746_Ala750del")
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

fn refseq_request() -> RequestedVariantIdentity {
    RequestedVariantIdentity {
        gene: Some("ATM".into()),
        coding_change: Some("c.1066-6T>G".into()),
        transcript: Some("NM_000051.4".into()),
        genomic_accession: Some("NC_000011.10".into()),
        genome_build: Some("GRCh38".into()),
        position: Some(108248927),
        reference: Some("T".into()),
        alternate: Some("G".into()),
        ..Default::default()
    }
}

fn refseq_hit(id: &str, gene: Option<&str>, coding: Option<&str>) -> MyVariantHit {
    serde_json::from_value(serde_json::json!({
        "_id": id,
        "dbnsfp": {
            "genename": gene,
            "hgvsc": coding
        }
    }))
    .expect("valid MyVariant hit")
}

fn scan(
    requested: &RequestedVariantIdentity,
    hits: Vec<MyVariantHit>,
    exhaustive: bool,
) -> ProviderScan {
    ProviderScan {
        candidates: hits
            .into_iter()
            .map(|hit| provider_candidate(requested, hit))
            .collect(),
        exhaustive,
        available: true,
    }
}

#[test]
fn article_provider_aggregation_follows_precedence_and_refseq_fallback_states() {
    let requested = refseq_request();
    let compatible = refseq_hit(
        "GRCh38:NC_000011.10:g.108248927T>G",
        Some("ATM"),
        Some("NM_000051.4:c.1066-6T>G"),
    );
    let contradictory = refseq_hit(
        "GRCh38:NC_000011.10:g.108248928T>G",
        Some("ATM"),
        Some("NM_000051.4:c.1066-6T>G"),
    );

    let confirmed = article_resolution_context(
        requested.clone(),
        scan(
            &requested,
            vec![contradictory.clone(), compatible.clone()],
            true,
        ),
    );
    assert_eq!(
        confirmed.resolution.provider_validation.status,
        VariantProviderValidationStatus::Confirmed
    );
    assert_eq!(
        confirmed.resolution.basis,
        Some(VariantArticleResolutionBasis::ProviderConfirmed)
    );
    assert_eq!(
        confirmed.source_hit.as_ref().map(|hit| hit.id.as_str()),
        Some("GRCh38:NC_000011.10:g.108248927T>G")
    );

    let indeterminate = article_resolution_context(
        requested.clone(),
        scan(&requested, vec![compatible.clone()], false),
    );
    assert_eq!(
        indeterminate.resolution.provider_validation.status,
        VariantProviderValidationStatus::Indeterminate
    );
    assert_eq!(
        indeterminate.resolution.status,
        VariantResolutionStatus::Resolved
    );

    let contradictory = article_resolution_context(
        requested.clone(),
        scan(&requested, vec![contradictory], true),
    );
    assert_eq!(
        contradictory.resolution.provider_validation.status,
        VariantProviderValidationStatus::Contradictory
    );
    assert_eq!(
        contradictory
            .resolution
            .provider_validation
            .contradictory_field
            .as_deref(),
        Some("position")
    );
    assert_eq!(
        contradictory.resolution.status,
        VariantResolutionStatus::Unresolved
    );

    let not_found = article_resolution_context(requested.clone(), scan(&requested, vec![], true));
    assert_eq!(
        not_found.resolution.provider_validation.status,
        VariantProviderValidationStatus::NotFound
    );
    assert_eq!(
        not_found.resolution.basis,
        Some(VariantArticleResolutionBasis::CallerSupplied)
    );

    let unavailable = article_resolution_context(
        requested,
        ProviderScan {
            candidates: Vec::new(),
            exhaustive: false,
            available: false,
        },
    );
    assert_eq!(
        unavailable.resolution.provider_validation.status,
        VariantProviderValidationStatus::Unavailable
    );
    assert!(unavailable.available);
}

#[test]
fn article_provider_aggregation_marks_distinct_compatible_and_indeterminate_sets() {
    let requested = refseq_request();
    let compatible = refseq_hit(
        "GRCh38:NC_000011.10:g.108248927T>G",
        Some("ATM"),
        Some("NM_000051.4:c.1066-6T>G"),
    );
    let compatible_with_rsid: MyVariantHit = serde_json::from_value(serde_json::json!({
        "_id": "GRCh38:NC_000011.10:g.108248927T>G",
        "dbnsfp": {
            "genename": "ATM",
            "hgvsc": "NM_000051.4:c.1066-6T>G"
        },
        "dbsnp": {"rsid": "rs605"}
    }))
    .expect("valid MyVariant hit");
    let multiple = article_resolution_context(
        requested.clone(),
        scan(
            &requested,
            vec![compatible.clone(), compatible_with_rsid],
            true,
        ),
    );
    assert_eq!(
        multiple.resolution.provider_validation.status,
        VariantProviderValidationStatus::Indeterminate
    );

    let missing_facts = refseq_hit("GRCh38:NC_000011.10:g.108248927T>G", None, None);
    let with_indeterminate = article_resolution_context(
        requested.clone(),
        scan(&requested, vec![compatible, missing_facts], true),
    );
    assert_eq!(
        with_indeterminate.resolution.provider_validation.status,
        VariantProviderValidationStatus::Indeterminate
    );

    let position_first = refseq_hit(
        "GRCh38:NC_000011.10:g.108248926T>G",
        Some("ATM"),
        Some("NM_000051.4:c.1066-6T>G"),
    );
    let gene_second = refseq_hit(
        "GRCh38:NC_000011.10:g.108248928T>G",
        Some("OTHER"),
        Some("NM_000051.4:c.1066-6T>G"),
    );
    let left = article_resolution_context(
        requested.clone(),
        scan(
            &requested,
            vec![gene_second.clone(), position_first.clone()],
            true,
        ),
    );
    let right = article_resolution_context(
        requested.clone(),
        scan(&requested, vec![position_first, gene_second], true),
    );
    assert_eq!(left.resolution, right.resolution);
    assert_eq!(
        left.resolution
            .provider_validation
            .contradictory_field
            .as_deref(),
        Some("position")
    );
}

#[test]
fn article_provider_aggregation_is_order_independent_and_selects_stable_alias() {
    let requested = RequestedVariantIdentity {
        coding_change: None,
        transcript: None,
        ..refseq_request()
    };
    let first = refseq_hit(
        "GRCh38:NC_000011.10:g.108248927T>G",
        Some("ATM"),
        Some("c.1066-6T>G"),
    );
    let second = refseq_hit(
        "GRCh38:NC_000011.10:g.108248927T>G",
        Some("ATM"),
        Some("NM_000051.4:c.1066-6T>G"),
    );
    let left = article_resolution_context(
        requested.clone(),
        scan(&requested, vec![first.clone(), second.clone()], true),
    );
    let right = article_resolution_context(
        requested.clone(),
        scan(&requested, vec![second, first], true),
    );
    assert_eq!(left.resolution, right.resolution);
    assert_eq!(
        left.resolution.provider_validation.matched_alias.as_deref(),
        Some("GRCh38:NC_000011.10:g.108248927T>G")
    );
}
