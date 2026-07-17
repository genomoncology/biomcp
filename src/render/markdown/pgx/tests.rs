use super::*;

#[test]
fn pgx_markdown_includes_evidence_links() {
    let pgx = Pgx {
        section_outcomes: crate::entities::pgx::default_pgx_section_outcomes(),
        query: "CYP2D6".to_string(),
        gene: Some("CYP2D6".to_string()),
        drug: Some("warfarin".to_string()),
        interactions: Vec::new(),
        recommendations: Vec::new(),
        frequencies: Vec::new(),
        guidelines: Vec::new(),
        annotations: Vec::new(),
        annotations_note: None,
    };

    let markdown = pgx_markdown(&pgx, &[]).expect("rendered markdown");
    assert!(markdown.contains("[CPIC](https://cpicpgx.org/genes/cyp2d6/)"));
    assert!(markdown.contains("[PharmGKB](https://www.pharmgkb.org/gene/CYP2D6)"));
    assert!(markdown.contains("[PharmGKB](https://www.pharmgkb.org/chemical/warfarin)"));
}
