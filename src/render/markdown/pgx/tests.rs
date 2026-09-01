use super::*;

fn empty_pgx(query: &str) -> Pgx {
    serde_json::from_value(serde_json::json!({"query": query})).expect("empty PGx")
}

#[test]
fn pgx_markdown_includes_evidence_links() {
    let mut pgx = empty_pgx("CYP2D6");
    pgx.gene = Some("CYP2D6".to_string());
    pgx.drug = Some("warfarin".to_string());

    let markdown = pgx_markdown(&pgx, &[]).expect("rendered markdown");
    assert!(markdown.contains("[CPIC](https://cpicpgx.org/genes/cyp2d6/)"));
    assert!(markdown.contains("[PharmGKB](https://www.pharmgkb.org/gene/CYP2D6)"));
    assert!(markdown.contains("[PharmGKB](https://www.pharmgkb.org/chemical/warfarin)"));
}

#[test]
fn recommendations_render_genotypes_and_page_drug_coverage() {
    let pgx: Pgx = serde_json::from_value(serde_json::json!({
        "query": "TPMT",
        "gene": "TPMT",
        "recommendations": [{
            "drugname": "azathioprine",
            "genotype": [
                ["TPMT", "Normal Metabolizer"],
                ["NUDT15", "Poor Metabolizer"]
            ],
            "recommendation": "Consider alternative nonthiopurine therapy.",
            "classification": "Strong"
        }],
        "recommendation_drugs": ["azathioprine", "mercaptopurine", "thioguanine"]
    }))
    .expect("PGx fixture");

    let markdown = pgx_markdown(&pgx, &["recommendations".into()]).expect("markdown");
    assert!(
        markdown.contains("| Drug | Genotype | Activity Score | Recommendation | Classification |")
    );
    assert!(markdown.contains(
        "| azathioprine | TPMT Normal Metabolizer; NUDT15 Poor Metabolizer | - | Consider alternative nonthiopurine therapy. | Strong |"
    ));
    assert!(markdown.contains(
        "Drugs on this page: azathioprine. Also held for TPMT: mercaptopurine, thioguanine."
    ));

    let json = serde_json::to_value(&pgx).expect("PGx JSON");
    assert_eq!(
        json["recommendation_drugs"],
        serde_json::json!(["azathioprine", "mercaptopurine", "thioguanine"])
    );
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
    let mut pgx = empty_pgx("CYP2D6");
    pgx.section_pagination = pagination;
    pgx.gene = Some("CYP2D6".into());

    let markdown = pgx_markdown(&pgx, &["recommendations".into()]).expect("markdown");
    assert!(!markdown.contains("## Interactions"));
    assert!(markdown.contains("recommendations --offset 10 --limit 10"));
}
