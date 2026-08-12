//! Tier 2 — request construction. Pure: builds the GraphQL `RequestPlan` and
//! asserts the exact method / path / JSON body that would be sent. Nothing is sent.

use super::super::*;
use crate::error::BioMcpError;
use crate::sources::{HttpMethod, RequestBody};

#[test]
fn gene_constraint_plan_posts_graphql_query_and_symbol() {
    let plan = GnomadClient::gene_constraint_plan(" TP53 ").unwrap();

    assert_eq!(plan.method, HttpMethod::Post);
    assert_eq!(plan.path, "");
    assert!(plan.query.is_empty());
    let RequestBody::Json(body) = &plan.body else {
        panic!("expected JSON body, got {:?}", plan.body);
    };
    assert!(body["query"].as_str().unwrap().contains("GeneConstraint"));
    assert_eq!(body["variables"]["symbol"], "TP53");
}

#[test]
fn gene_constraint_plan_rejects_invalid_gene_symbols() {
    for symbol in ["", "TP 53", "TP53/ALK"] {
        assert!(
            matches!(
                GnomadClient::gene_constraint_plan(symbol),
                Err(BioMcpError::InvalidArgument(_))
            ),
            "expected invalid argument for {symbol:?}"
        );
    }
}

#[test]
fn variant_population_plan_pins_gnomad_v4_and_complete_fields() {
    let plan = GnomadClient::variant_population_plan(" 10-112998590-C-T ").unwrap();

    assert_eq!(plan.method, HttpMethod::Post);
    assert_eq!(plan.path, "");
    let RequestBody::Json(body) = &plan.body else {
        panic!("expected JSON body, got {:?}", plan.body);
    };
    assert_eq!(body["variables"]["variantId"], "10-112998590-C-T");
    let query = body["query"].as_str().unwrap();
    for required in [
        "VariantPopulation",
        "dataset: gnomad_r4",
        "variant_id",
        "exome",
        "genome",
        "ac",
        "an",
        "homozygote_count",
        "hemizygote_count",
        "filters",
        "faf95",
        "popmax",
        "popmax_population",
        "populations",
    ] {
        assert!(query.contains(required), "missing {required} from {query}");
    }
    assert_eq!(GNOMAD_VARIANT_MAX_BODY_BYTES, 512 * 1024);
}

#[test]
fn variant_population_plan_rejects_blank_or_oversized_ids() {
    for variant_id in ["".to_string(), "x".repeat(257)] {
        assert!(matches!(
            GnomadClient::variant_population_plan(&variant_id),
            Err(BioMcpError::InvalidArgument(_))
        ));
    }
}
