//! Original synthetic cases exercise ordinary callable rsID detail retrieval.
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use crate::entities::variant;
use crate::error::BioMcpError;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

pub(super) const AMBIGUOUS: &str =
    "RsID lookup is ambiguous. Use an exact genomic HGVS ID, or search with the rsID.";
const EVIDENCE: &str = "RsID lookup lacks complete identity evidence. Use an exact genomic HGVS ID, or search with the rsID.";
const INCOMPLETE: &str = "RsID lookup did not complete its candidate scan. Use an exact genomic HGVS ID, or search with the rsID.";

pub(super) fn hit() -> Value {
    json!({"_id":"chr1:g.101A>T", "dbsnp":{"rsid":"rs101"},
        "dbnsfp":{"genename":["GENE"], "hgvsp":["p.Ala11Val"], "hgvsc":["NM_000001.1:c.31A>T"]}})
}
pub(super) fn detail() -> Value {
    json!({"section_outcomes":{
        "cancerhotspots":{"outcome":"not_requested","sources":[]},
        "cbioportal":{"outcome":"not_requested","sources":[]},
        "civic":{"outcome":"not_requested","sources":[]},
        "clinvar":{"outcome":"not_requested","sources":[]},
        "gwas":{"outcome":"not_requested","sources":[]},
        "population":{"outcome":"not_requested","sources":[]},
        "predict":{"outcome":"not_requested","sources":[]}},
        "gene":"GENE", "id":"chr1:g.101A>T", "genome_build":"GRCh37",
        "genome_build_provenance":"MyVariant.info provider default",
        "rsid":"rs101"})
}
pub(super) fn api_error() -> Value {
    json!({"error":{"code":"api","message":"API request to MyVariant.info failed.",
        "source":"MyVariant.info","recovery":"Retry the remote source."},"_meta":{"not_found":false}})
}
pub(super) async fn fixture(pages: Vec<Value>) -> (TestHttpFixture, Arc<Mutex<Vec<String>>>) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    let fixture = TestHttpFixture::spawn(move |request| {
        let mut log = captured.lock().unwrap();
        let index = log.len();
        log.push(request.into());
        let (status, bytes) = match pages.get(index) {
            Some(page) if page == "failure" => {
                ("400 Bad Request", b"provider-body-secret".to_vec())
            }
            Some(page) => ("200 OK", serde_json::to_vec(page).unwrap()),
            None => ("400 Bad Request", b"unexpected-request".to_vec()),
        };
        TestHttpReply::Bytes(test_http_response(status, "application/json", &bytes))
    })
    .await;
    (fixture, requests)
}
pub(super) fn ledger(requests: &Arc<Mutex<Vec<String>>>, offsets: &[usize], size: usize) {
    let log = requests.lock().unwrap();
    assert_eq!(log.len(), offsets.len(), "{log:?}");
    for (request, offset) in log.iter().zip(offsets) {
        let words = request
            .lines()
            .next()
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>();
        assert_eq!((words[0], words[2]), ("GET", "HTTP/1.1"));
        let url = reqwest::Url::parse(&format!("http://localhost{}", words[1])).unwrap();
        assert_eq!(url.path(), "/query");
        assert_eq!(
            url.query_pairs().into_owned().collect::<Vec<_>>(),
            vec![
                ("q".into(), "dbsnp.rsid:rs101".into()),
                ("size".into(), size.to_string()),
                ("from".into(), offset.to_string()),
                ("fields".into(), FIELDS.into())
            ]
        );
        assert_eq!(request.split_once("\r\n\r\n").unwrap().1, "");
    }
}
async fn exercise(
    pages: Vec<Value>,
    outcome: &str,
    offsets: &[usize],
    size: usize,
    expected: Value,
) {
    let (fixture, requests) = fixture(pages).await;
    let mut env = TestEnv::new();
    env.set("BIOMCP_MYVARIANT_BASE", &fixture.base);
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_CACHE_MODE", "off");
    let result = variant::get("rs101", &[]).await;
    // Selection is asserted first so a request-size change cannot manufacture baseline red.
    match outcome {
        "success" => assert_eq!(serde_json::to_value(result.unwrap()).unwrap(), expected),
        "absent" => match result.unwrap_err() {
            BioMcpError::NotFound {
                entity,
                id,
                suggestion,
            } => {
                assert_eq!(
                    (entity.as_str(), id.as_str(), suggestion.as_str()),
                    (
                        "variant",
                        "rs101",
                        "Try searching: biomcp search variant -g \"rs101\""
                    )
                );
            }
            error => panic!("{error:?}"),
        },
        "failure" => assert_eq!(
            serde_json::from_str::<Value>(
                &crate::render::json::to_error_json(&result.unwrap_err()).unwrap()
            )
            .unwrap(),
            api_error()
        ),
        refusal => match result.unwrap_err() {
            BioMcpError::InvalidArgument(message) => assert_eq!(
                message,
                match refusal {
                    "ambiguous" => AMBIGUOUS,
                    "evidence" => EVIDENCE,
                    "incomplete" => INCOMPLETE,
                    _ => panic!("unknown outcome"),
                }
            ),
            error => panic!("{error:?}"),
        },
    }
    ledger(&requests, offsets, size);
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn rsid_get_keeps_unique_detail() {
    exercise(
        vec![json!({"total":1,"hits":[hit()]})],
        "success",
        &[0],
        50,
        detail(),
    )
    .await;
}
#[tokio::test]
#[serial_test::serial(source_env)]
async fn rsid_get_continues_past_contradictory_rows() {
    let mut wrong = hit();
    wrong["dbsnp"]["rsid"] = json!("rs202");
    exercise(
        vec![
            json!({"total":11,"hits":vec![wrong;10]}),
            json!({"total":11,"hits":[hit()]}),
        ],
        "success",
        &[0, 10],
        50,
        detail(),
    )
    .await;
}
#[tokio::test]
#[serial_test::serial(source_env)]
async fn rsid_get_refuses_distinct_source_identities_despite_annotation_scores() {
    let mut other = hit();
    other["_id"] = json!("chr1:g.102A>T");
    other["cadd"] = json!({"phred":30});
    exercise(
        vec![json!({"total":2,"hits":[hit(),other]})],
        "ambiguous",
        &[0],
        50,
        detail(),
    )
    .await;
}
#[tokio::test]
#[serial_test::serial(source_env)]
async fn rsid_get_preserves_opaque_source_id_case() {
    let mut a = hit();
    a["_id"] = json!("opaque-a");
    let mut b = a.clone();
    b["_id"] = json!("OPAQUE-A");
    exercise(
        vec![json!({"total":2,"hits":[a,b]})],
        "ambiguous",
        &[0],
        50,
        detail(),
    )
    .await;
}
#[tokio::test]
#[serial_test::serial(source_env)]
async fn rsid_get_distinguishes_absence_from_unusable_evidence() {
    let mut wrong = hit();
    wrong["dbsnp"]["rsid"] = json!("rs202");
    exercise(
        vec![json!({"total":1,"hits":[wrong.clone()]})],
        "absent",
        &[0],
        50,
        detail(),
    )
    .await;
    for rsid in [Value::Null, json!(""), json!("   ")] {
        let mut missing = hit();
        missing["_id"] = json!("chr1:g.104A>T");
        missing["dbsnp"]["rsid"] = rsid;
        exercise(
            vec![json!({"total":2,"hits":[hit(),missing]})],
            "evidence",
            &[0],
            50,
            detail(),
        )
        .await;
    }
    for id in ["", "   "] {
        let mut blank = hit();
        blank["_id"] = json!(id);
        exercise(
            vec![json!({"total":1,"hits":[blank]})],
            "evidence",
            &[0],
            50,
            detail(),
        )
        .await;
    }
    exercise(
        vec![json!({"total":2,"hits":[wrong,hit()]})],
        "success",
        &[0],
        50,
        detail(),
    )
    .await;
}
#[tokio::test]
#[serial_test::serial(source_env)]
async fn rsid_get_deduplicates_assertion_order_and_retains_first_display_order() {
    let mut rich = hit();
    rich["cadd"] = json!({"phred":30});
    exercise(
        vec![json!({"total":2,"hits":[hit(),rich]})],
        "success",
        &[0],
        50,
        detail(),
    )
    .await;
    let mut first = hit();
    first["dbnsfp"]["hgvsp"] = json!(["p.Ala11Val", "p.Ser20dup"]);
    first["dbnsfp"]["hgvsc"] = json!(["NM_000001.1:c.31A>T", "NM_000001.1:c.60_62dup"]);
    first["snpeff"] = json!({"ann":[
        {"feature_id":"NM_000001.1","genename":"GENE","hgvs_c":"c.31A>T","hgvs_p":"p.Ala11Val"},
        {"feature_id":"NM_000001.1","genename":"GENE","hgvs_c":"c.60_62dup","hgvs_p":"p.Ser20dup"}]});
    let mut second = first.clone();
    second["snpeff"]["ann"].as_array_mut().unwrap().reverse();
    second["dbnsfp"]["hgvsp"] = json!(["p.Ser20dup", "p.Ala11Val"]);
    second["dbnsfp"]["hgvsc"] = json!(["NM_000001.1:c.60_62dup", "NM_000001.1:c.31A>T"]);
    let mut displayed = detail();
    displayed["hgvs_p"] = json!("p.Ala11Val");
    displayed["hgvs_c"] = json!("c.31A>T");
    displayed["transcript"] = json!("NM_000001.1");
    displayed["legacy_name"] = json!("GENE A11V");
    exercise(
        vec![json!({"total":2,"hits":[first,second]})],
        "success",
        &[0],
        50,
        displayed,
    )
    .await;
}
#[tokio::test]
#[serial_test::serial(source_env)]
async fn rsid_get_keeps_verbatim_assertions_and_multiplicity_distinct() {
    let mut prefixed = hit();
    prefixed["dbnsfp"]["hgvsc"] = json!(["NM_000001.1:c.31A>T", "c.31A>T"]);
    let mut repeated = prefixed.clone();
    repeated["dbnsfp"]["hgvsc"] = json!(["NM_000001.1:c.31A>T", "c.31A>T", "c.31A>T"]);
    for (a, b) in [(hit(), prefixed.clone()), (prefixed, repeated)] {
        exercise(
            vec![json!({"total":2,"hits":[a,b]})],
            "ambiguous",
            &[0],
            50,
            detail(),
        )
        .await;
    }
}
#[tokio::test]
#[serial_test::serial(source_env)]
async fn rsid_get_requires_empty_page_without_a_total() {
    exercise(
        vec![json!({"hits":[hit()]}), json!({"hits":[]})],
        "success",
        &[0, 1],
        50,
        detail(),
    )
    .await;
}
#[tokio::test]
#[serial_test::serial(source_env)]
async fn rsid_get_retains_maximum_total_and_refuses_early_empty_pages() {
    for end in [
        json!({"total":2,"hits":[]}),
        json!({"hits":[]}),
        json!({"total":1,"hits":[]}),
    ] {
        exercise(
            vec![json!({"total":2,"hits":[hit()]}), end],
            "incomplete",
            &[0, 1],
            50,
            detail(),
        )
        .await;
    }
    exercise(
        vec![json!({"total":2,"hits":[]})],
        "incomplete",
        &[0],
        50,
        detail(),
    )
    .await;
    exercise(
        vec![
            json!({"total":3,"hits":[hit()]}),
            json!({"total":2,"hits":[hit()]}),
            json!({"hits":[]}),
        ],
        "incomplete",
        &[0, 1, 2],
        50,
        detail(),
    )
    .await;
}
#[tokio::test]
#[serial_test::serial(source_env)]
async fn rsid_get_distinguishes_exact_bound_from_unfinished_scans() {
    for (total, outcome) in [
        (Some(1000), "success"),
        (Some(1001), "incomplete"),
        (None, "incomplete"),
    ] {
        let pages = (0..20)
            .map(|_| {
                let mut page = json!({"hits":vec![hit();50]});
                if let Some(total) = total {
                    page["total"] = json!(total);
                }
                page
            })
            .collect();
        exercise(
            pages,
            outcome,
            &(0..20).map(|n| n * 50).collect::<Vec<_>>(),
            50,
            detail(),
        )
        .await;
    }
}
#[tokio::test]
#[serial_test::serial(source_env)]
async fn rsid_get_refuses_received_but_unexamined_rows_before_ambiguity() {
    let mut rows = vec![hit(); 1001];
    rows[1]["_id"] = json!("chr1:g.102A>T");
    rows[1000]["_id"] = json!("chr1:g.103A>T");
    exercise(
        vec![json!({"total":1000,"hits":rows})],
        "incomplete",
        &[0],
        50,
        detail(),
    )
    .await;
}
#[tokio::test]
#[serial_test::serial(source_env)]
async fn rsid_get_delivers_later_source_failure_before_provisional_detail() {
    exercise(
        vec![json!({"total":2,"hits":[hit()]}), json!("failure")],
        "failure",
        &[0, 1],
        50,
        detail(),
    )
    .await;
}

const FIELDS: &str = "_id,cadd.phred,cadd.consequence,clinvar.gene.symbol,clinvar.rcv.accession,clinvar.rcv.version,clinvar.rcv.clinical_significance,clinvar.rcv.review_status,clinvar.rcv.conditions,clinvar.rcv.preferred_name,clinvar.rcv.last_evaluated,clinvar.rcv.number_submitters,clinvar.variant_id,snpeff.ann.feature_id,snpeff.ann.genename,snpeff.ann.hgvs_c,snpeff.ann.hgvs_p,dbnsfp.genename,dbnsfp.hgvsp,dbnsfp.hgvsc,dbnsfp.sift.pred,dbnsfp.sift.score,dbnsfp.polyphen2.hdiv.pred,dbnsfp.revel.score,dbnsfp.revel.rankscore,dbnsfp.alphamissense.score,dbnsfp.alphamissense.pred,dbnsfp.alphamissense.rankscore,dbnsfp.clinpred.score,dbnsfp.clinpred.pred,dbnsfp.metarnn.score,dbnsfp.metarnn.pred,dbnsfp.bayesdel.add_af.score,dbnsfp.bayesdel.add_af.pred,dbnsfp.bayesdel.no_af.score,dbnsfp.bayesdel.no_af.pred,dbnsfp.phylop.100way_vertebrate.rankscore,dbnsfp.phylop.470way_mammalian.rankscore,dbnsfp.phastcons.100way_vertebrate.rankscore,dbnsfp.phastcons.470way_mammalian.rankscore,dbnsfp.gerp++.rs,dbsnp.rsid,gnomad_exome.af.af,gnomad_exome.af.af_afr,gnomad_exome.af.af_eas,gnomad_exome.af.af_nfe,gnomad_exome.af.af_sas,gnomad_exome.af.af_amr,gnomad_exome.af.af_asj,gnomad_exome.af.af_fin,gnomad_exome.af.af_afr_female,gnomad_exome.af.af_afr_male,gnomad_exome.af.af_amr_female,gnomad_exome.af.af_amr_male,gnomad_exome.af.af_eas_jpn,gnomad_exome.af.af_eas_kor,gnomad_exome.af.af_nfe_bgr,gnomad_exome.af.af_nfe_est,gnomad_exome.af.af_nfe_nwe,gnomad_exome.af.af_nfe_onf,gnomad_exome.af.af_nfe_seu,gnomad_exome.af.af_nfe_swe,gnomad_exome.af.af_oth,gnomad.exomes.af.af,gnomad.exomes.af.af_afr,gnomad.exomes.af.af_eas,gnomad.exomes.af.af_nfe,gnomad.exomes.af.af_sas,gnomad.exomes.af.af_amr,gnomad.exomes.af.af_asj,gnomad.exomes.af.af_fin,gnomad.genomes.af.af,gnomad.genomes.af.af_afr,gnomad.genomes.af.af_eas,gnomad.genomes.af.af_nfe,gnomad.genomes.af.af_sas,gnomad.genomes.af.af_amr,gnomad.genomes.af.af_asj,gnomad.genomes.af.af_fin,exac.af,exac_nontcga.af,cosmic.cosmic_id,cosmic.mut_freq,cosmic.tumor_site,cosmic.mut_nt,cgi,civic";
