use super::*;

#[test]
fn pgx_markdown_includes_evidence_links() {
    let pgx = Pgx {
        section_pagination: std::collections::BTreeMap::new(),
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

#[test]
fn recommendations_only_hides_interactions_and_advertises_its_offset() {
    let mut pagination = std::collections::BTreeMap::new();
    pagination.insert(
        "recommendations".into(),
        crate::entities::pgx::PgxSectionPagination {
            offset: 0,
            limit: 10,
            returned: 10,
            total: Some(12),
            has_more: true,
            next_offset: Some(10),
        },
    );
    let pgx = Pgx {
        section_pagination: pagination,
        section_outcomes: crate::entities::pgx::default_pgx_section_outcomes(),
        query: "CYP2D6".into(),
        gene: Some("CYP2D6".into()),
        drug: None,
        interactions: Vec::new(),
        recommendations: Vec::new(),
        frequencies: Vec::new(),
        guidelines: Vec::new(),
        annotations: Vec::new(),
        annotations_note: None,
    };

    let markdown = pgx_markdown(&pgx, &["recommendations".into()]).expect("markdown");
    assert!(!markdown.contains("## Interactions"));
    assert!(markdown.contains("recommendations --offset 10 --limit 10"));
}
