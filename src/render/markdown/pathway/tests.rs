use super::*;

#[test]
fn pathway_markdown_uses_source_and_source_specific_evidence_url() {
    let pathway = Pathway {
        section_outcomes: Default::default(),
        source: "KEGG".to_string(),
        id: "hsa05200".to_string(),
        name: "Pathways in cancer".to_string(),
        species: Some("Homo sapiens".to_string()),
        summary: Some("Cancer pathway overview.".to_string()),
        genes: vec!["BRAF".to_string(), "EGFR".to_string()],
        events: Vec::new(),
        enrichment: Vec::new(),
    };

    let markdown = pathway_markdown(&pathway, &[]).expect("rendered markdown");
    assert!(markdown.contains("Source: KEGG"));
    assert!(markdown.contains("[KEGG](https://www.kegg.jp/entry/hsa05200)"));

    let wikipathways = Pathway {
        section_outcomes: Default::default(),
        source: "WikiPathways".to_string(),
        id: "WP254".to_string(),
        name: "Apoptosis".to_string(),
        species: Some("Homo sapiens".to_string()),
        summary: None,
        genes: vec!["TP53".to_string()],
        events: Vec::new(),
        enrichment: Vec::new(),
    };

    let markdown = pathway_markdown(&wikipathways, &[]).expect("rendered markdown");
    assert!(markdown.contains("Source: WikiPathways"));
    assert!(markdown.contains("[WikiPathways](https://www.wikipathways.org/pathways/WP254.html)"));
}

#[test]
fn pathway_markdown_hides_genes_section_when_genes_are_empty() {
    let pathway = Pathway {
        section_outcomes: Default::default(),
        source: "KEGG".to_string(),
        id: "hsa05200".to_string(),
        name: "Pathways in cancer".to_string(),
        species: Some("Homo sapiens".to_string()),
        summary: Some("Cancer pathway overview.".to_string()),
        genes: Vec::new(),
        events: Vec::new(),
        enrichment: Vec::new(),
    };

    let markdown = pathway_markdown(&pathway, &[]).expect("rendered markdown");
    assert!(!markdown.contains("## Genes"));
    assert!(!markdown.contains("BRAF"));
}

#[test]
fn pathway_search_markdown_shows_source_column() {
    let results = vec![
        PathwaySearchResult {
            source: "Reactome".to_string(),
            id: "R-HSA-5673001".to_string(),
            name: "RAF/MAP kinase cascade".to_string(),
        },
        PathwaySearchResult {
            source: "KEGG".to_string(),
            id: "hsa04010".to_string(),
            name: "MAPK signaling pathway".to_string(),
        },
    ];

    let markdown = pathway_search_markdown_with_footer("MAPK", &results, Some(results.len()), "")
        .expect("markdown");
    assert!(markdown.contains("| Source | ID | Name |"));
    assert!(markdown.contains("| Reactome | R-HSA-5673001 | RAF/MAP kinase cascade |"));
    assert!(markdown.contains("| KEGG | hsa04010 | MAPK signaling pathway |"));
}
