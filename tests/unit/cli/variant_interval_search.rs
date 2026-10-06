//! Full accepted 0675 routing objects and bounded original-input witnesses.
use super::*;
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use crate::error::BioMcpError;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
fn gold() -> Value {
    serde_json::from_str(include_str!(
        "../../../src/entities/variant/resolution/interval_search_oracles.json"
    ))
    .unwrap()
}
fn expand(v: &Value) -> String {
    if let Some(s) = v.as_str() {
        return s.to_owned();
    }
    if let Some(parts) = v["concat"].as_array() {
        return parts.iter().map(expand).collect();
    }
    v["repeat"]
        .as_str()
        .unwrap()
        .repeat(v["count"].as_u64().unwrap() as usize)
}
fn optional(v: &Value) -> Option<String> {
    v.as_str().map(str::to_owned)
}
fn tokens(v: &Value) -> Vec<String> {
    v.as_array().unwrap().iter().map(expand).collect()
}
fn plan_value(p: &VariantSearchPlan) -> Value {
    match p {
        VariantSearchPlan::Standard(q) => json!({"variant":"Standard","query":{
            "gene":q.gene,"hgvsp":q.hgvsp,"hgvsc":q.hgvsc,"rsid":q.rsid,
            "protein_alias":q.protein_alias,"consequence":q.consequence,"condition":q.condition,
            "requested_identity":q.requested_identity.as_ref().map(|i|json!({
                "gene":i.gene,"protein_change":i.protein_change,"coding_change":i.coding_change,
                "transcript":i.transcript,"genomic_accession":i.genomic_accession,"genome_build":i.genome_build,
                "position":i.position,"reference":i.reference,"alternate":i.alternate,"rsid":i.rsid}))}}),
        VariantSearchPlan::GeneFirstCandidate {
            gene,
            condition,
            hgvsp,
            consequence,
        } => json!({"variant":"GeneFirstCandidate","gene":gene,"condition":condition,
                "hgvsp":hgvsp,"consequence":consequence}),
        VariantSearchPlan::Guidance(g) => json!({"variant":"Guidance","guidance":{
            "query":g.query,"kind":g.kind,"next_commands":g.next_commands}}),
    }
}
fn assert_error(error: &BioMcpError, expected: &Value) {
    let BioMcpError::InvalidArgument(message) = error else {
        panic!("expected typed InvalidArgument, got {error:?}");
    };
    assert_eq!(message, expected["message"].as_str().unwrap());
    assert_eq!(expected["type"], "BioMcpError::InvalidArgument");
    if expected.get("public_cli_json").is_some() {
        let mut public =
            serde_json::from_str::<Value>(&crate::render::json::to_error_json(error).unwrap())
                .unwrap();
        public["results"] = json!([]);
        assert_eq!(public, expected["public_cli_json"]);
    }
}
fn args(v: &Value) -> VariantSearchArgs {
    VariantSearchArgs {
        gene: optional(&v["gene"]),
        positional_query: tokens(&v["positional_query"]),
        hgvsp: optional(&v["hgvsp"]),
        significance: optional(&v["significance"]),
        max_frequency: v["max_frequency"].as_f64(),
        min_cadd: v["min_cadd"].as_f64(),
        consequence: optional(&v["consequence"]),
        review_status: optional(&v["review_status"]),
        population: optional(&v["population"]),
        revel_min: v["revel_min"].as_f64(),
        gerp_min: v["gerp_min"].as_f64(),
        tumor_site: optional(&v["tumor_site"]),
        condition: optional(&v["condition"]),
        impact: optional(&v["impact"]),
        lof: v["lof"].as_bool().unwrap(),
        has: optional(&v["has"]),
        missing: optional(&v["missing"]),
        therapy: optional(&v["therapy"]),
        limit: v["limit"].as_u64().unwrap() as usize,
        offset: v["offset"].as_u64().unwrap() as usize,
    }
}
async fn bounded(row: &Value) {
    let seam = &row["bounded_search_seam"];
    let ledger = Arc::new(Mutex::new(Vec::<String>::new()));
    let captured = Arc::clone(&ledger);
    let fixture = TestHttpFixture::spawn(move |request| {
        captured.lock().unwrap().push(request.to_owned());
        TestHttpReply::Bytes(test_http_response(
            "200 OK",
            "application/json",
            br#"{"total":0,"hits":[]}"#,
        ))
    })
    .await;
    let cache = tempfile::tempdir().unwrap();
    let mut env = TestEnv::new();
    env.set("BIOMCP_MYVARIANT_BASE", &fixture.base);
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_CACHE_DIR", cache.path());
    env.set("BIOMCP_CACHE_MODE", "off");
    let result = handle_search_bounded(
        args(&seam["args"]),
        seam["json"].as_bool().unwrap(),
        seam["alias_suggestions_as_json"].as_bool().unwrap(),
    )
    .await;
    // Capture unexpected requests before the terminal assertion so red retains the ledger.
    let requests = ledger.lock().unwrap().clone();
    assert_eq!(
        json!(requests),
        seam["expected_requests"],
        "{} original-input ledger",
        row["id"]
    );
    if seam["expected_result"]["kind"] == "Err" {
        let error = result.expect_err("terminal refusal has no outcome");
        assert_error(
            error.downcast_ref::<BioMcpError>().unwrap(),
            &seam["expected_result"],
        );
    } else {
        let outcome = result.unwrap();
        assert_eq!(
            outcome.exit_code,
            seam["expected_result"]["exit_code"].as_u64().unwrap() as u8
        );
        assert_eq!(outcome.stream, crate::cli::OutputStream::Stdout);
        assert_eq!(
            serde_json::from_str::<Value>(&outcome.text).unwrap(),
            seam["expected_result"]["decoded_stdout"]
        );
        assert!(seam["expected_result"]["stderr"].is_null());
    }
}
#[tokio::test]
#[serial_test::parallel(source_env)]
async fn interval_search_routing_table() {
    let gold = gold();
    let rows = gold["routing_cases"].as_array().unwrap();
    assert_eq!(rows.len(), 22);
    for (index, row) in rows.iter().enumerate() {
        assert_eq!(row["id"], format!("IR{:02}", index + 1));
        let input = &row["input"];
        let expected = &row["expected"];
        let result = if input["entrypoint"] == "parse_variant_id" {
            crate::entities::variant::parse_variant_id(input["id"].as_str().unwrap())
                .map(|_| panic!("exact get remains refused"))
        } else {
            query::resolve_variant_query(
                optional(&input["gene_flag"]),
                optional(&input["hgvsp_flag"]),
                optional(&input["consequence_flag"]),
                optional(&input["condition_flag"]),
                tokens(&input["positional_tokens"]),
            )
        };
        if expected["variant"] == "Error" {
            assert_error(&result.unwrap_err(), expected);
        } else {
            let plan = result.unwrap();
            // IR16 checks final refusal semantics for the original three-word input.
            // It remains outside interval admission and keeps the complete fallback oracle.
            let plan = if row["id"] == "IR16" {
                let VariantSearchPlan::GeneFirstCandidate {
                    gene,
                    condition,
                    hgvsp,
                    consequence,
                } = plan
                else {
                    panic!("IR16 must reach gene confirmation without interval selection");
                };
                let (resolved, _) =
                    query::apply_gene_first_routing(gene, condition, None, hgvsp, consequence);
                VariantSearchPlan::Standard(resolved)
            } else {
                plan
            };
            let actual = plan_value(&plan);
            let mut expected_plan = expected.clone();
            for key in [
                "native_guidance_json",
                "public_cli_json",
                "public_cli_exit",
                "public_cli_stderr",
            ] {
                expected_plan.as_object_mut().unwrap().remove(key);
            }
            assert_eq!(actual, expected_plan, "{} entire plan", row["id"]);
            if let VariantSearchPlan::Guidance(g) = plan {
                let native = serde_json::from_str::<Value>(
                    &crate::render::json::to_variant_guidance_json(&g).unwrap(),
                )
                .unwrap();
                assert_eq!(native, expected["native_guidance_json"]);
                let mut public = native;
                public["results"] = json!([]);
                assert_eq!(public, expected["public_cli_json"]);
            }
        }
        if row.get("bounded_search_seam").is_some() {
            bounded(row).await;
        }
    }
    // Nonexact gene operands retain condition fallback even beside qualified interval text.
    for source in [
        "gene NP_1:p.A11del",
        "GENE_OTHER NP_1:p.A11del",
        "gene NP_1:p.A11del extra",
    ] {
        let plan =
            query::resolve_variant_query(None, None, None, None, vec![source.to_owned()]).unwrap();
        assert_eq!(
            plan,
            VariantSearchPlan::standard(ResolvedVariantQuery {
                condition: Some(source.to_owned()),
                ..Default::default()
            })
        );
    }
}
