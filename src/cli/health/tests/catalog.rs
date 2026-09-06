//! Catalog tests for `biomcp health` source descriptors.

use super::super::catalog::{ProbeKind, affects_for_api, health_sources};
#[test]
fn health_inventory_includes_all_expected_sources() {
    let names: Vec<_> = health_sources().iter().map(|source| source.api).collect();

    assert_eq!(
        names,
        vec![
            "GenCC",
            "MyGene",
            "MyVariant",
            "Cancerhotspots.org",
            "MyChem",
            "PubTator3",
            "PubMed",
            "Europe PMC",
            "NCBI E-utilities",
            "LitSense2",
            "PMC OA",
            "NCBI ID Converter",
            "ClinicalTrials.gov",
            "NCI CTS",
            "Enrichr",
            "OpenFDA",
            "CDC WONDER VAERS",
            "OncoKB",
            "DisGeNET",
            "AlphaGenome",
            "Semantic Scholar",
            "Figshare",
            "CPIC",
            "PharmGKB",
            "Monarch",
            "HPO",
            "MyDisease",
            "SEER Explorer",
            "NIH Reporter",
            "CIViC",
            "GWAS Catalog",
            "GTEx",
            "DGIdb",
            "ClinGen",
            "ClinGen Allele Registry",
            "ClinGen CSpec",
            "ClinGen ERepo",
            "ClinGen LDH",
            "dbSNP",
            "gnomAD",
            "UniProt",
            "QuickGO",
            "STRING",
            "Reactome",
            "KEGG",
            "WikiPathways",
            "g:Profiler",
            "OpenTargets",
            "ChEMBL",
            "HPA",
            "InterPro",
            "ComplexPortal",
            "OLS4",
            "UMLS",
            "MedlinePlus",
            "cBioPortal",
        ]
    );
}

#[test]
fn gencc_health_uses_dedicated_head_contract() {
    let source = health_sources()
        .iter()
        .find(|source| source.api == "GenCC")
        .expect("GenCC health source");
    assert!(matches!(source.probe, ProbeKind::GenCcHead));
    assert_eq!(source.affects, Some("gene gencc section"));
}

#[test]
fn pharmgkb_health_row_probes_and_explains_the_clinpgx_move() {
    let source = health_sources()
        .iter()
        .find(|source| source.api == "PharmGKB")
        .expect("PharmGKB health source");

    let ProbeKind::Get { url } = source.probe else {
        panic!("PharmGKB health source should use a GET probe");
    };

    assert!(url.starts_with("https://api.clinpgx.org/v1/data/labelAnnotation?"));
    let affects = source.affects.expect("PharmGKB regression note");
    assert!(affects.contains("get pgx") && affects.contains("annotations"));
    assert!(affects.contains("api.pharmgkb.org") && affects.contains("api.clinpgx.org"));
}

#[test]
fn nci_health_probe_uses_keyword_query() {
    let source = health_sources()
        .iter()
        .find(|source| source.api == "NCI CTS")
        .expect("nci health source");

    let ProbeKind::AuthGet { url, .. } = source.probe else {
        panic!("NCI CTS health source should use an authenticated GET probe");
    };

    assert!(url.contains("keyword=melanoma"));
    assert!(!url.contains("diseases=melanoma"));
}

#[test]
fn clingen_health_descriptors_and_affects_remain_distinct() {
    let descriptors = health_sources()
        .iter()
        .filter(|source| source.api.starts_with("ClinGen "))
        .map(|source| (source.api, source.affects))
        .collect::<Vec<_>>();

    for descriptor in [
        ("ClinGen Allele Registry", Some("variant normalization")),
        ("ClinGen CSpec", Some("gene cspec helper")),
        ("ClinGen ERepo", Some("variant erepo helper")),
        ("ClinGen LDH", Some("variant article identity verification")),
    ] {
        assert!(
            descriptors.contains(&descriptor),
            "missing ClinGen health descriptor: {descriptor:?}"
        );
    }
}

#[test]
fn markdown_shows_new_affects_mappings() {
    assert_eq!(affects_for_api("GTEx"), Some("gene expression section"));
    assert_eq!(affects_for_api("DGIdb"), Some("gene druggability section"));
    assert_eq!(
        affects_for_api("OpenTargets"),
        Some("gene druggability, drug target, and disease association sections")
    );
    assert_eq!(affects_for_api("ClinGen"), Some("gene clingen section"));
    assert_eq!(
        affects_for_api("ClinGen Allele Registry"),
        Some("variant normalization")
    );
    assert_eq!(affects_for_api("ClinGen CSpec"), Some("gene cspec helper"));
    assert_eq!(
        affects_for_api("ClinGen ERepo"),
        Some("variant erepo helper")
    );
    assert_eq!(
        affects_for_api("ClinGen LDH"),
        Some("variant article identity verification")
    );
    assert_eq!(
        affects_for_api("dbSNP"),
        Some("variant population coordinate resolution")
    );
    assert_eq!(affects_for_api("gnomAD"), Some("gene constraint section"));
    assert_eq!(
        affects_for_api("NIH Reporter"),
        Some("gene and disease funding sections")
    );
    assert_eq!(
        affects_for_api("KEGG"),
        Some("pathway search and detail sections")
    );
    assert_eq!(
        affects_for_api("HPA"),
        Some("gene protein tissue expression and localization section")
    );
    assert_eq!(
        affects_for_api("ComplexPortal"),
        Some("protein complex membership section")
    );
    assert_eq!(
        affects_for_api("g:Profiler"),
        Some("gene enrichment (biomcp enrich)")
    );
    assert_eq!(
        affects_for_api("Figshare"),
        Some("non-PMC article asset fallback")
    );
}
