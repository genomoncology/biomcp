//! Cross-cutting markdown tests for the markdown module facade and shared outputs.

use super::*;
use crate::entities::adverse_event::DeviceEvent;
use crate::entities::article::{Article, ArticleAnnotations};
use crate::entities::drug::Drug;
use crate::entities::gene::Gene;
use crate::entities::pathway::Pathway;
use crate::entities::pgx::Pgx;
use crate::entities::variant::Variant;

#[test]
fn proof_markdown_module_layout_uses_directory_module() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let markdown_dir = root.join("src/render/markdown");

    assert!(
        markdown_dir.is_dir(),
        "expected src/render/markdown/ to exist"
    );
    assert!(
        markdown_dir.join("mod.rs").is_file(),
        "expected src/render/markdown/mod.rs to exist"
    );
    assert!(
        markdown_dir.join("tests.rs").is_file(),
        "expected src/render/markdown/tests.rs to exist"
    );
    assert!(
        markdown_dir.join("root_tests.rs").is_file(),
        "expected src/render/markdown/root_tests.rs to exist"
    );
    assert!(
        markdown_dir.join("test_support.rs").is_file(),
        "expected src/render/markdown/test_support.rs to exist"
    );
    assert!(
        !root.join("src/render/markdown.rs").exists(),
        "expected src/render/markdown.rs to be removed"
    );
}

#[test]
fn markdown_detail_outputs_label_gene_drug_and_disease_sources() {
    let gene = Gene {
        symbol: "CFTR".to_string(),
        name: "CF transmembrane conductance regulator".to_string(),
        entrez_id: "1080".to_string(),
        ensembl_id: None,
        location: Some("7q31.2".to_string()),
        genomic_coordinates: None,
        omim_id: None,
        uniprot_id: Some("P13569".to_string()),
        summary: Some("Chloride channel.".to_string()),
        gene_type: Some("protein-coding".to_string()),
        aliases: vec!["ABCC7".to_string()],
        clinical_diseases: Vec::new(),
        clinical_drugs: Vec::new(),
        pathways: None,
        ontology: Some(vec![crate::entities::gene::EnrichmentResult {
            library: "GO_Biological_Process_2025".to_string(),
            terms: vec![crate::entities::gene::EnrichmentTerm {
                name: "ion transport".to_string(),
                p_value: 0.001,
                genes: "CFTR".to_string(),
            }],
        }]),
        diseases: Some(vec![crate::entities::gene::EnrichmentResult {
            library: "DisGeNET".to_string(),
            terms: vec![crate::entities::gene::EnrichmentTerm {
                name: "cystic fibrosis".to_string(),
                p_value: 0.002,
                genes: "CFTR".to_string(),
            }],
        }]),
        protein: Some(crate::entities::gene::GeneProtein {
            accession: "P13569".to_string(),
            name: "CFTR".to_string(),
            function: Some("ATP-binding cassette transporter.".to_string()),
            length: Some(1480),
            isoforms: Vec::new(),
            alternative_names: Vec::new(),
        }),
        go: Some(vec![crate::entities::gene::GeneGoTerm {
            id: "GO:0006811".to_string(),
            name: "ion transport".to_string(),
            aspect: Some("BP".to_string()),
            evidence: Some("EXP".to_string()),
        }]),
        interactions: Some(vec![crate::entities::gene::GeneInteraction {
            partner: "SLC26A9".to_string(),
            score: Some(0.83),
        }]),
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
    let gene_markdown = gene_markdown(&gene, &[]).expect("gene markdown");
    assert!(gene_markdown.contains("Source: NCBI Gene / MyGene.info"));
    assert!(gene_markdown.contains("## Summary (NCBI Gene)"));
    assert!(gene_markdown.contains("## Aliases (NCBI Gene / MyGene.info)"));
    assert!(gene_markdown.contains("## Ontology (Enrichr)"));
    assert!(gene_markdown.contains("## Diseases (Enrichr)"));
    assert!(gene_markdown.contains("## Protein (UniProt)"));
    assert!(gene_markdown.contains("## GO Terms (QuickGO)"));
    assert!(gene_markdown.contains("## Interactions (STRING)"));
    assert!(gene_markdown.contains("See also:"));
    assert!(gene_markdown.contains("biomcp search variant -g CFTR"));

    let drug = Drug {
        name: "ivacaftor".to_string(),
        drugbank_id: Some("DB08820".to_string()),
        chembl_id: Some("CHEMBL1200749".to_string()),
        unii: None,
        drug_type: Some("small molecule".to_string()),
        mechanism: Some("CFTR potentiator".to_string()),
        mechanisms: vec!["Potentiates CFTR chloride transport.".to_string()],
        approval_date: Some("2012-01-31".to_string()),
        approval_date_raw: Some("2012-01-31".to_string()),
        approval_date_display: Some("January 31, 2012".to_string()),
        approval_summary: Some("FDA approved on January 31, 2012".to_string()),
        brand_names: vec!["Kalydeco".to_string()],
        route: Some("Oral".to_string()),
        targets: vec!["CFTR".to_string()],
        variant_targets: Vec::new(),
        target_family: None,
        target_family_name: None,
        indications: vec!["Cystic fibrosis".to_string()],
        interactions: vec![crate::entities::drug::DrugInteraction {
            drug: "rifampin".to_string(),
            level: None,
            description: Some("May reduce ivacaftor exposure.".to_string()),
            partner_classes: Vec::new(),
        }],
        interaction_text: None,
        pharm_classes: Vec::new(),
        top_adverse_events: vec!["Cough".to_string()],
        faers_query: None,
        label: None,
        label_set_id: None,
        shortage: Some(Vec::new()),
        approvals: Some(Vec::new()),
        us_safety_warnings: None,
        ema_regulatory: None,
        ema_safety: None,
        ema_shortage: None,
        who_prequalification: None,
        civic: None,
    };
    let drug_markdown = drug_markdown(&drug, &["all".to_string()]).expect("drug markdown");
    assert!(drug_markdown.contains("Type (MyChem.info): small molecule"));
    assert!(drug_markdown.contains("FDA Approved (DrugCentral): January 31, 2012"));
    assert!(drug_markdown.contains("Brand Names (DrugBank): Kalydeco"));
    assert!(drug_markdown.contains("Safety (OpenFDA FAERS): Cough"));
    assert!(drug_markdown.contains("## Mechanisms (MyChem.info / ChEMBL)"));
    assert!(drug_markdown.contains("## Targets (ChEMBL / Open Targets)"));
    assert!(drug_markdown.contains("## Indications (Open Targets)"));
    assert!(drug_markdown.contains("## Interactions (DDInter)"));
    assert!(drug_markdown.contains("## Shortage (US - OpenFDA Drug Shortages)"));
    assert!(drug_markdown.contains("## Regulatory (US - Drugs@FDA)"));

    let disease = crate::entities::disease::Disease {
        id: "MONDO:0009061".to_string(),
        name: "cystic fibrosis".to_string(),
        definition: Some("Inherited disease affecting chloride transport.".to_string()),
        synonyms: vec!["CF".to_string()],
        parents: vec!["autosomal recessive disease".to_string()],
        associated_genes: vec!["CFTR".to_string()],
        gene_associations: Vec::new(),
        top_genes: vec!["CFTR".to_string()],
        top_gene_scores: Vec::new(),
        treatment_landscape: vec!["ivacaftor".to_string()],
        recruiting_trial_count: Some(4),
        pathways: vec![crate::entities::disease::DiseasePathway {
            id: "R-HSA-5673001".to_string(),
            name: "Ion channel transport".to_string(),
        }],
        phenotypes: Vec::new(),
        clinical_features: Vec::new(),
        key_features: Vec::new(),
        variants: Vec::new(),
        top_variant: None,
        models: Vec::new(),
        prevalence: Vec::new(),
        prevalence_note: None,
        survival: None,
        survival_note: None,
        civic: None,
        disgenet: None,
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
        xrefs: std::collections::HashMap::new(),
    };
    let disease_markdown =
        disease_markdown(&disease, &["all".to_string()]).expect("disease markdown");
    assert!(disease_markdown.contains("## Definition (MyDisease.info)"));
    assert!(disease_markdown.contains("Inherited disease affecting chloride transport."));
    assert!(disease_markdown.contains("Genes (Open Targets): CFTR"));
    assert!(disease_markdown.contains("Treatments (MyChem.info indication search): ivacaftor"));
    assert!(disease_markdown.contains("Recruiting Trials (ClinicalTrials.gov): 4"));
    assert!(disease_markdown.contains("## Synonyms (MONDO / Disease Ontology via MyDisease.info)"));
    assert!(disease_markdown.contains("## Parents (MONDO / Disease Ontology via MyDisease.info)"));
    assert!(disease_markdown.contains("## Pathways (Reactome)"));
}

#[test]
fn markdown_detail_outputs_label_article_trial_and_pathway_sources() {
    let article = Article {
        pmid: Some("22663011".to_string()),
        pmcid: Some("PMC9984800".to_string()),
        doi: Some("10.1000/example".to_string()),
        title: "Example article".to_string(),
        authors: vec!["A. Author".to_string()],
        journal: Some("Example Journal".to_string()),
        date: Some("2012-05-31".to_string()),
        citation_count: Some(12),
        publication_type: Some("Journal Article".to_string()),
        open_access: Some(true),
        abstract_text: Some("Abstract text.".to_string()),
        full_text_path: None,
        full_text_note: Some("Saved full text unavailable.".to_string()),
        full_text_source: None,
        annotations: Some(ArticleAnnotations {
            genes: vec![AnnotationCount {
                text: "CFTR".to_string(),
                count: 1,
            }],
            diseases: Vec::new(),
            chemicals: Vec::new(),
            mutations: Vec::new(),
        }),
        semantic_scholar: Some(crate::entities::article::ArticleSemanticScholar {
            paper_id: Some("paper-1".to_string()),
            tldr: Some("TLDR".to_string()),
            citation_count: Some(10),
            influential_citation_count: Some(2),
            reference_count: Some(5),
            is_open_access: Some(true),
            open_access_pdf: None,
        }),
        pubtator_fallback: false,
    };
    let article_markdown = article_markdown(&article, &["all".to_string()]).expect("article");
    assert!(article_markdown.contains("Source: PubMed / Europe PMC"));
    assert!(article_markdown.contains("## Authors (PubMed / Europe PMC)"));
    assert!(article_markdown.contains("## Abstract (PubMed / Europe PMC)"));
    assert!(article_markdown.contains("## PubTator Annotations"));
    assert!(article_markdown.contains("## Full Text"));
    assert!(!article_markdown.contains("## Full Text (PMC OA)"));
    assert!(article_markdown.contains("## Semantic Scholar"));

    let trial = crate::entities::trial::Trial {
        nct_id: "NCT06668103".to_string(),
        source: Some("ClinicalTrials.gov".to_string()),
        title: "Example trial".to_string(),
        status: "Recruiting".to_string(),
        phase: Some("Phase 2".to_string()),
        study_type: Some("Interventional".to_string()),
        age_range: Some("18 Years and older".to_string()),
        conditions: vec!["cystic fibrosis".to_string()],
        interventions: vec!["ivacaftor".to_string()],
        intervention_details: Vec::new(),
        sponsor: Some("Example Sponsor".to_string()),
        enrollment: Some(42),
        summary: Some("Trial summary.".to_string()),
        start_date: Some("2025-01-01".to_string()),
        completion_date: None,
        eligibility_text: Some("Eligibility text.".to_string()),
        locations: Some(vec![crate::entities::trial::TrialLocation {
            facility: "Example Hospital".to_string(),
            city: "Boston".to_string(),
            state: Some("MA".to_string()),
            country: "United States".to_string(),
            status: Some("Recruiting".to_string()),
            contact_name: None,
            contact_phone: None,
        }]),
        outcomes: Some(crate::entities::trial::TrialOutcomes {
            primary: vec![crate::entities::trial::TrialOutcome {
                measure: "FEV1".to_string(),
                description: None,
                time_frame: None,
            }],
            secondary: Vec::new(),
        }),
        arms: Some(vec![crate::entities::trial::TrialArm {
            label: "Arm A".to_string(),
            arm_type: Some("Experimental".to_string()),
            description: Some("Description".to_string()),
            interventions: vec!["ivacaftor".to_string()],
        }]),
        references: Some(vec![crate::entities::trial::TrialReference {
            pmid: Some("22663011".to_string()),
            citation: "Example citation".to_string(),
            reference_type: Some("background".to_string()),
        }]),
    };
    let trial_markdown = trial_markdown(&trial, &["all".to_string()]).expect("trial");
    assert!(trial_markdown.contains("Source: ClinicalTrials.gov"));
    assert!(trial_markdown.contains("## Conditions (ClinicalTrials.gov)"));
    assert!(trial_markdown.contains("## Interventions (ClinicalTrials.gov)"));
    assert!(trial_markdown.contains("## Summary (ClinicalTrials.gov)"));
    assert!(trial_markdown.contains("## Eligibility (ClinicalTrials.gov)"));
    assert!(trial_markdown.contains("## Locations (ClinicalTrials.gov)"));
    assert!(trial_markdown.contains("## Outcomes (ClinicalTrials.gov)"));
    assert!(trial_markdown.contains("## Arms (ClinicalTrials.gov)"));
    assert!(trial_markdown.contains("## References (ClinicalTrials.gov)"));

    let pathway = Pathway {
        source: "Reactome".to_string(),
        id: "R-HSA-5358351".to_string(),
        name: "Signal transduction".to_string(),
        species: Some("Homo sapiens".to_string()),
        summary: Some("Reactome summary.".to_string()),
        genes: vec!["CFTR".to_string()],
        events: vec!["Channel gating".to_string()],
        enrichment: vec![crate::entities::pathway::PathwayEnrichment {
            source: "Reactome".to_string(),
            id: "R-HSA-1234".to_string(),
            name: "Transport".to_string(),
            p_value: Some(0.001),
        }],
    };
    let pathway_markdown = pathway_markdown(&pathway, &["all".to_string()]).expect("pathway");
    assert!(pathway_markdown.contains("## Summary (Reactome)"));
    assert!(pathway_markdown.contains("## Genes (Reactome)"));
    assert!(pathway_markdown.contains("## Events (Reactome)"));
    assert!(pathway_markdown.contains("## Enrichment (g:Profiler)"));
}

#[test]
fn markdown_detail_outputs_label_variant_protein_pgx_and_openfda_sources() {
    let variant: Variant = serde_json::from_value(serde_json::json!({
        "id": "rs334",
        "gene": "HBB",
        "hgvs_p": "p.Glu7Val",
        "rsid": "rs334",
        "prediction": {
            "expression_lfc": 0.3,
            "splice_score": 0.1,
            "chromatin_score": 0.2,
            "top_gene": "HBB"
        },
        "gnomad_af": 0.01,
        "conservation": {
            "phylop_100way_vertebrate": 0.8
        },
        "expanded_predictions": [
            {"tool": "REVEL", "score": 0.91, "prediction": "Damaging"}
        ],
        "cgi_associations": [
            {"drug": "hydroxyurea", "association": "responsive"}
        ]
    }))
    .expect("variant");
    let variant_markdown = variant_markdown(&variant, &["all".to_string()]).expect("variant");
    assert!(variant_markdown.contains("Source: MyVariant.info / ClinVar"));
    assert!(variant_markdown.contains("## AlphaGenome Prediction"));
    assert!(variant_markdown.contains("## Population (gnomAD via MyVariant.info)"));
    assert!(variant_markdown.contains("## Conservation (MyVariant.info)"));
    assert!(variant_markdown.contains("## Expanded Predictions (MyVariant.info)"));
    assert!(variant_markdown.contains("## CGI Drug Associations (Cancer Genome Interpreter)"));

    let protein = crate::entities::protein::Protein {
        accession: "P15056".to_string(),
        entry_id: Some("BRAF_HUMAN".to_string()),
        name: "Serine/threonine-protein kinase B-raf".to_string(),
        gene_symbol: Some("BRAF".to_string()),
        organism: Some("Homo sapiens".to_string()),
        length: Some(766),
        function: Some("Kinase function.".to_string()),
        structures: vec!["6V34".to_string()],
        structure_count: Some(1),
        domains: vec![crate::entities::protein::ProteinDomain {
            accession: "IPR000719".to_string(),
            name: Some("Protein kinase domain".to_string()),
            domain_type: Some("domain".to_string()),
        }],
        interactions: vec![crate::entities::protein::ProteinInteraction {
            partner: "MEK1".to_string(),
            score: Some(0.92),
        }],
        complexes: vec![crate::entities::protein::ProteinComplex {
            accession: "CPX-1".to_string(),
            name: "BRAF complex".to_string(),
            description: None,
            curation: crate::entities::protein::ProteinComplexCuration::Curated,
            components: vec![crate::entities::protein::ProteinComplexComponent {
                accession: "P15056".to_string(),
                name: "BRAF".to_string(),
                stoichiometry: None,
            }],
        }],
    };
    let protein_markdown = protein_markdown(&protein, &["all".to_string()]).expect("protein");
    assert!(protein_markdown.contains("Source: UniProt"));
    assert!(protein_markdown.contains("## Function (UniProt)"));
    assert!(protein_markdown.contains("## Structures (PDB / AlphaFold via UniProt)"));
    assert!(protein_markdown.contains("## Domains (InterPro)"));
    assert!(protein_markdown.contains("## Interactions (STRING)"));
    assert!(protein_markdown.contains("## Complexes (ComplexPortal)"));

    let pgx = Pgx {
        query: "CYP2D6".to_string(),
        gene: Some("CYP2D6".to_string()),
        drug: Some("codeine".to_string()),
        interactions: vec![crate::entities::pgx::PgxInteraction {
            genesymbol: "CYP2D6".to_string(),
            drugname: "codeine".to_string(),
            cpiclevel: Some("A".to_string()),
            pgxtesting: Some("Recommended".to_string()),
            guidelinename: None,
            guidelineurl: None,
        }],
        recommendations: vec![crate::entities::pgx::PgxRecommendation {
            drugname: "codeine".to_string(),
            phenotype: Some("Poor metabolizer".to_string()),
            activity_score: None,
            implication: None,
            recommendation: Some("Avoid codeine".to_string()),
            classification: Some("Strong".to_string()),
            population: None,
            guidelinename: None,
            guidelineurl: None,
        }],
        frequencies: vec![crate::entities::pgx::PgxFrequency {
            genesymbol: "CYP2D6".to_string(),
            allele: "*4".to_string(),
            population_group: Some("European".to_string()),
            subject_count: None,
            frequency: None,
            min_frequency: None,
            max_frequency: None,
        }],
        guidelines: vec![crate::entities::pgx::PgxGuideline {
            name: "CPIC Guideline".to_string(),
            url: Some("https://example.org/guideline".to_string()),
            genes: vec!["CYP2D6".to_string()],
            drugs: vec!["codeine".to_string()],
        }],
        annotations: Vec::new(),
        annotations_note: Some("PharmGKB note.".to_string()),
    };
    let pgx_markdown = pgx_markdown(&pgx, &["all".to_string()]).expect("pgx");
    assert!(pgx_markdown.contains("Source: CPIC"));
    assert!(pgx_markdown.contains("## Interactions (CPIC)"));
    assert!(pgx_markdown.contains("## Recommendations (CPIC)"));
    assert!(pgx_markdown.contains("## Population Frequencies (CPIC)"));
    assert!(pgx_markdown.contains("| CYP2D6 | *4 | European | - | - |"));
    assert!(pgx_markdown.contains("## Guidelines (CPIC)"));

    let faers = crate::entities::adverse_event::AdverseEvent {
        report_id: "10329882".to_string(),
        drug: "ivacaftor".to_string(),
        reactions: vec!["Cough".to_string()],
        outcomes: vec!["Hospitalization".to_string()],
        patient: None,
        concomitant_medications: vec!["azithromycin".to_string()],
        reporter_type: None,
        reporter_country: None,
        indication: None,
        serious: true,
        date: Some("2024-01-01".to_string()),
    };
    let faers_markdown = adverse_event_markdown(&faers, &["all".to_string()]).expect("faers");
    assert!(faers_markdown.contains("Source: OpenFDA"));
    assert!(faers_markdown.contains("## Reactions (OpenFDA)"));
    assert!(faers_markdown.contains("## Outcomes (OpenFDA)"));
    assert!(faers_markdown.contains("## Concomitant Drugs (OpenFDA)"));

    let device = DeviceEvent {
        report_id: "MDR-123".to_string(),
        report_number: None,
        device: "Infusion Pump".to_string(),
        manufacturer: Some("Example".to_string()),
        event_type: Some("Malfunction".to_string()),
        date: Some("2024-02-01".to_string()),
        description: Some("Description text.".to_string()),
    };
    let device_markdown = device_event_markdown(&device).expect("device");
    assert!(device_markdown.contains("Source: OpenFDA"));
    assert!(device_markdown.contains("## Description (OpenFDA)"));
}

#[test]
fn pagination_footer_offset_suppresses_more_when_complete_single_result() {
    let footer = pagination_footer(PaginationFooterMode::Offset, 0, 10, 1, Some(1), None);
    assert!(footer.contains("Showing 1 of 1 results."));
    assert!(!footer.contains("Use --offset"));
}

#[test]
fn pagination_footer_offset_keeps_more_when_additional_rows_exist() {
    let footer = pagination_footer(PaginationFooterMode::Offset, 0, 2, 2, Some(10), None);
    assert!(footer.contains("Showing 1-2 of 10 results."));
    assert!(footer.contains("Use --offset 2 for more."));
}

#[test]
fn pagination_footer_offset_suppresses_more_on_last_page() {
    let footer = pagination_footer(PaginationFooterMode::Offset, 8, 2, 2, Some(10), None);
    assert!(footer.contains("Showing 9-10 of 10 results."));
    assert!(!footer.contains("Use --offset"));
}

#[test]
fn pagination_footer_cursor_prefers_offset_guidance_without_placeholder() {
    let footer = pagination_footer(
        PaginationFooterMode::Cursor,
        0,
        1,
        1,
        Some(20),
        Some("abc123"),
    );
    assert!(footer.contains("Use --offset 1 for more."));
    assert!(footer.contains("--next-page is also supported"));
    assert!(!footer.contains("<TOKEN>"));
}
