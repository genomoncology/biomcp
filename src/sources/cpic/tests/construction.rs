//! Tier 2 - request construction. Pure: builds CPIC `RequestPlan`s and asserts
//! the exact path / query that would be sent. Nothing is sent.

use super::super::*;
use crate::error::BioMcpError;
use crate::sources::HttpMethod;

#[test]
fn pair_plans_set_expected_filters_and_order() {
    let gene = CpicClient::pairs_by_gene_plan(" cyp2d6 ", 250, 10).expect("gene pair plan");
    assert_eq!(gene.method, HttpMethod::Get);
    assert_eq!(gene.path, "pair_view");
    assert_eq!(gene.query_value("genesymbol"), Some("eq.CYP2D6"));
    assert_eq!(gene.query_value("limit"), Some("200"));
    assert_eq!(gene.query_value("offset"), Some("10"));
    assert_eq!(
        gene.query_value("order"),
        Some("cpiclevel.asc,drugname.asc")
    );

    let drug = CpicClient::pairs_by_drug_plan(" clop*idogrel% ", 5, 2).expect("drug pair plan");
    assert_eq!(drug.path, "pair_view");
    assert_eq!(drug.query_value("drugname"), Some("ilike.*clopidogrel*"));
    assert_eq!(drug.query_value("limit"), Some("5"));
    assert_eq!(drug.query_value("offset"), Some("2"));
    assert_eq!(
        drug.query_value("order"),
        Some("cpiclevel.asc,genesymbol.asc")
    );
}

#[test]
fn recommendation_frequency_and_guideline_plans_set_expected_filters() {
    let rec_gene = CpicClient::recommendations_by_gene_plan("cyp2d6", 3).expect("gene rec plan");
    assert_eq!(rec_gene.path, "recommendation_view");
    assert_eq!(
        rec_gene.query_value("lookupkey->>CYP2D6"),
        Some("not.is.null")
    );

    let rec_drug = CpicClient::recommendations_by_drug_plan("codeine", 3).expect("drug rec plan");
    assert_eq!(rec_drug.path, "recommendation_view");
    assert_eq!(rec_drug.query_value("drugname"), Some("ilike.*codeine*"));

    let freq = CpicClient::frequencies_by_gene_plan("cyp2d6", 3).expect("frequency plan");
    assert_eq!(freq.path, "population_frequency_view");
    assert_eq!(freq.query_value("genesymbol"), Some("eq.CYP2D6"));

    let guide = CpicClient::guidelines_by_gene_plan("cyp2d6", 3).expect("guideline plan");
    assert_eq!(guide.path, "guideline_summary_view");
    assert_eq!(
        guide.query_value("genes"),
        Some("cs.[{\"symbol\":\"CYP2D6\"}]")
    );
}

#[test]
fn plans_validate_gene_and_drug_inputs() {
    assert!(matches!(
        CpicClient::pairs_by_gene_plan("CYP2D6!", 5, 0),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        CpicClient::pairs_by_drug_plan(" ", 5, 0),
        Err(BioMcpError::InvalidArgument(_))
    ));
}
