//! Catalog tests for `biomcp health` source descriptors.

use super::super::catalog::{ProbeKind, affects_for_api, health_sources};
#[test]
fn health_inventory_includes_all_expected_sources() {
    let names: Vec<_> = health_sources().iter().map(|source| source.api).collect();

    assert_eq!(
        names,
        vec![
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
fn markdown_shows_new_affects_mappings() {
    assert_eq!(affects_for_api("GTEx"), Some("gene expression section"));
    assert_eq!(affects_for_api("DGIdb"), Some("gene druggability section"));
    assert_eq!(
        affects_for_api("OpenTargets"),
        Some("gene druggability, drug target, and disease association sections")
    );
    assert_eq!(affects_for_api("ClinGen"), Some("gene clingen section"));
    assert_eq!(affects_for_api("ClinGen CSpec"), Some("gene cspec helper"));
    assert_eq!(
        affects_for_api("ClinGen ERepo"),
        Some("variant erepo helper")
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
