use super::*;

#[test]
fn protein_markdown_renders_complexes_summary_and_detail_bullets() {
    let protein = Protein {
            section_outcomes: Default::default(),
            accession: "P15056".to_string(),
            entry_id: Some("BRAF_HUMAN".to_string()),
            name: "Serine/threonine-protein kinase B-raf".to_string(),
            gene_symbol: Some("BRAF".to_string()),
            organism: Some("Homo sapiens".to_string()),
            length: Some(766),
            function: None,
            structures: Vec::new(),
            structure_count: None,
            domains: Vec::new(),
            interactions: Vec::new(),
            complexes: vec![
                ProteinComplex {
                    accession: "CPX-1234".to_string(),
                    name: "BRAF signaling complex with an intentionally long display label that should truncate in the summary table".to_string(),
                    description: Some("Signals through MAPK.".to_string()),
                    curation: ProteinComplexCuration::Curated,
                    components: vec![
                        ProteinComplexComponent {
                            accession: "P15056".to_string(),
                            name: "BRAF".to_string(),
                            stoichiometry: Some("1".to_string()),
                        },
                        ProteinComplexComponent {
                            accession: "Q02750".to_string(),
                            name: "MAP2K1".to_string(),
                            stoichiometry: None,
                        },
                        ProteinComplexComponent {
                            accession: "P10398".to_string(),
                            name: "RAF1".to_string(),
                            stoichiometry: None,
                        },
                        ProteinComplexComponent {
                            accession: "P07900".to_string(),
                            name: "HSP90AA1".to_string(),
                            stoichiometry: None,
                        },
                        ProteinComplexComponent {
                            accession: "Q16543".to_string(),
                            name: "CDC37".to_string(),
                            stoichiometry: None,
                        },
                        ProteinComplexComponent {
                            accession: "Q9Y243".to_string(),
                            name: "AKT3".to_string(),
                            stoichiometry: None,
                        },
                        ProteinComplexComponent {
                            accession: "P31749".to_string(),
                            name: "AKT1".to_string(),
                            stoichiometry: None,
                        },
                    ],
                },
                ProteinComplex {
                    accession: "CPX-5678".to_string(),
                    name: "Complex with no listed members".to_string(),
                    description: None,
                    curation: ProteinComplexCuration::Predicted,
                    components: Vec::new(),
                },
            ],
        };

    let markdown = protein_markdown(&protein, &["complexes".to_string()]).expect("markdown");
    assert!(markdown.contains("## Complexes"));
    assert!(markdown.contains("| ID | Name | Members | Curation |"));
    assert!(!markdown.contains("| ID | Name | Components | Curation |"));
    assert!(
        markdown.contains(
            "| CPX-1234 | BRAF signaling complex with an intentionally long di… | 7 | curated |"
        ),
        "expected truncated complex summary row, got:\n{markdown}"
    );
    assert!(markdown.contains(
            "- `CPX-1234` members (7): BRAF (1), MAP2K1, RAF1, HSP90AA1, CDC37, +2 more\n  Description: Signals through MAPK."
        ));
    assert!(markdown.contains("- `CPX-5678` members (0): none listed"));
    assert!(!markdown.contains("- `CPX-1234`: Signals through MAPK."));
    assert!(!markdown.contains("AKT3"));
    assert!(!markdown.contains("AKT1"));
    assert!(!markdown.contains("See also: biomcp get protein P15056 complexes"));
}
