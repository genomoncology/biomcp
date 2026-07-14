use super::*;
use crate::entities::article::{AnnotationCount, Article, ArticleAnnotations};
use crate::entities::gene::Gene;

#[test]
fn gene_json_next_commands_parse() {
    let gene = Gene {
        symbol: "BRAF".to_string(),
        name: "B-Raf proto-oncogene".to_string(),
        entrez_id: "673".to_string(),
        ensembl_id: Some("ENSG00000157764".to_string()),
        location: Some("7q34".to_string()),
        genomic_coordinates: None,
        omim_id: Some("164757".to_string()),
        uniprot_id: Some("P15056".to_string()),
        summary: None,
        gene_type: None,
        aliases: Vec::new(),
        clinical_diseases: Vec::new(),
        clinical_drugs: Vec::new(),
        pathways: None,
        ontology: None,
        diseases: None,
        protein: None,
        go: None,
        interactions: None,
        civic: None,
        expression: None,
        hpa: None,
        druggability: None,
        clingen: None,
        constraint: None,
        disgenet: None,
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
    };

    let next_commands = crate::render::markdown::gene_next_commands(&gene, &[]);
    assert_eq!(
        next_commands
            .iter()
            .take(4)
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec![
            "biomcp get gene BRAF pathways",
            "biomcp get gene BRAF ontology",
            "biomcp get gene BRAF diseases",
            "biomcp get gene BRAF diagnostics",
        ]
    );
    assert!(
        next_commands.contains(&"biomcp search pgx -g BRAF".to_string()),
        "expected gene cross-entity helper after section follow-ups: {next_commands:?}"
    );
    assert!(next_commands.contains(&"biomcp search diagnostic --gene BRAF".to_string()));

    assert_entity_json_next_commands(
        "gene",
        &gene,
        crate::render::markdown::gene_evidence_urls(&gene),
        next_commands,
        crate::render::provenance::gene_section_sources(&gene),
    );
}

#[test]
fn gene_json_next_commands_omit_requested_section_follow_up() {
    let gene = Gene {
        symbol: "BRAF".to_string(),
        name: "B-Raf proto-oncogene".to_string(),
        entrez_id: "673".to_string(),
        ensembl_id: Some("ENSG00000157764".to_string()),
        location: Some("7q34".to_string()),
        genomic_coordinates: None,
        omim_id: Some("164757".to_string()),
        uniprot_id: Some("P15056".to_string()),
        summary: None,
        gene_type: None,
        aliases: Vec::new(),
        clinical_diseases: Vec::new(),
        clinical_drugs: Vec::new(),
        pathways: None,
        ontology: None,
        diseases: None,
        protein: None,
        go: None,
        interactions: None,
        civic: None,
        expression: None,
        hpa: None,
        druggability: None,
        clingen: None,
        constraint: None,
        disgenet: None,
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
    };

    let requested_sections = ["funding".to_string()];
    let next_commands = crate::render::markdown::gene_next_commands(&gene, &requested_sections);
    let json = crate::render::json::to_entity_json(
        &gene,
        crate::render::markdown::gene_evidence_urls(&gene),
        next_commands,
        crate::render::provenance::gene_section_sources(&gene),
    )
    .expect("gene json");
    let commands = collect_next_commands(&json);

    assert!(
        !commands.contains(&"biomcp get gene BRAF funding".to_string()),
        "requested section should not be suggested again: {commands:?}"
    );
}

#[test]
fn gene_json_suggestions_match_see_also_without_section_hints() {
    let gene = Gene {
        symbol: "BRAF".to_string(),
        name: "B-Raf proto-oncogene".to_string(),
        entrez_id: "673".to_string(),
        ensembl_id: Some("ENSG00000157764".to_string()),
        location: Some("7q34".to_string()),
        genomic_coordinates: None,
        omim_id: Some("164757".to_string()),
        uniprot_id: Some("P15056".to_string()),
        summary: None,
        gene_type: None,
        aliases: Vec::new(),
        clinical_diseases: Vec::new(),
        clinical_drugs: Vec::new(),
        pathways: None,
        ontology: None,
        diseases: None,
        protein: None,
        go: None,
        interactions: None,
        civic: None,
        expression: None,
        hpa: None,
        druggability: None,
        clingen: None,
        constraint: None,
        disgenet: None,
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
    };

    let json = crate::render::json::to_entity_json_with_suggestions(
        &gene,
        crate::render::markdown::gene_evidence_urls(&gene),
        crate::render::markdown::gene_next_commands(&gene, &[]),
        crate::render::markdown::related_gene(&gene),
        crate::render::provenance::gene_section_sources(&gene),
    )
    .expect("gene json");
    let suggestions = collect_suggestions(&json);

    assert!(suggestions.contains(&"biomcp search pgx -g BRAF".to_string()));
    assert!(!suggestions.contains(&"biomcp get gene BRAF pathways".to_string()));
}

#[test]
fn gene_json_next_commands_include_clingen_trial_search() {
    let gene = Gene {
        symbol: "SCN1A".to_string(),
        name: "sodium voltage-gated channel alpha subunit 1".to_string(),
        entrez_id: "6323".to_string(),
        ensembl_id: Some("ENSG00000144285".to_string()),
        location: Some("2q24.3".to_string()),
        genomic_coordinates: None,
        omim_id: Some("182389".to_string()),
        uniprot_id: Some("P35498".to_string()),
        summary: None,
        gene_type: None,
        aliases: Vec::new(),
        clinical_diseases: Vec::new(),
        clinical_drugs: Vec::new(),
        pathways: None,
        ontology: None,
        diseases: None,
        protein: None,
        go: None,
        interactions: None,
        civic: None,
        expression: None,
        hpa: None,
        druggability: None,
        clingen: Some(crate::sources::clingen::GeneClinGen {
            validity: vec![crate::sources::clingen::ClinGenValidity {
                disease: "genetic developmental and epileptic encephalopathy".to_string(),
                classification: "Definitive".to_string(),
                review_date: Some("2025-12-16".to_string()),
                moi: Some("AD".to_string()),
            }],
            haploinsufficiency: None,
            triplosensitivity: None,
        }),
        constraint: None,
        disgenet: None,
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
    };

    let next_commands = crate::render::markdown::gene_next_commands(&gene, &[]);
    let json = crate::render::json::to_entity_json(
        &gene,
        crate::render::markdown::gene_evidence_urls(&gene),
        next_commands,
        crate::render::provenance::gene_section_sources(&gene),
    )
    .expect("gene json");
    assert_json_next_commands_parse("gene-clingen", &json);
    assert!(collect_next_commands(&json).contains(
        &"biomcp search trial -c \"genetic developmental and epileptic encephalopathy\" -s recruiting"
            .to_string()
    ));
}

#[test]
fn article_json_next_commands_parse() {
    let article = Article {
        pmid: Some("22663011".to_string()),
        pmcid: Some("PMC9984800".to_string()),
        doi: Some("10.1056/NEJMoa1203421".to_string()),
        title: "Example about melanoma".to_string(),
        authors: Vec::new(),
        author_count: 0,
        author_completeness: crate::entities::article::ArticleAuthorCompleteness::Unavailable,
        author_source: crate::entities::article::ArticleSource::PubTator,
        journal: None,
        date: None,
        citation_count: None,
        publication_type: None,
        open_access: None,
        abstract_text: None,
        full_text_path: None,
        full_text_note: None,
        full_text_source: None,
        full_text_manifest: None,
        not_included: None,
        europepmc_license: None,
        europepmc_retracted: None,
        annotations: Some(ArticleAnnotations {
            genes: vec![AnnotationCount {
                text: "serine-threonine protein kinase".to_string(),
                count: 1,
            }],
            diseases: vec![AnnotationCount {
                text: "melanoma".to_string(),
                count: 1,
            }],
            chemicals: vec![AnnotationCount {
                text: "osimertinib".to_string(),
                count: 1,
            }],
            mutations: Vec::new(),
        }),
        semantic_scholar: None,
        pubtator_fallback: false,
    };
    let next_commands = crate::render::markdown::related_article(&article);
    assert!(
        next_commands
            .iter()
            .any(|cmd| { cmd == "biomcp search gene -q \"serine-threonine protein kinase\"" })
    );
    assert!(
        !next_commands
            .iter()
            .any(|cmd| cmd == "biomcp get gene serine-threonine protein kinase")
    );

    assert_entity_json_next_commands(
        "article",
        &article,
        crate::render::markdown::article_evidence_urls(&article),
        next_commands,
        crate::render::provenance::article_section_sources(&article),
    );
}
