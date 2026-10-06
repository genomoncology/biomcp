//! Original synthetic MIT examples test ordinary callable detail retrieval.
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use crate::entities::variant;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use tracing::instrument::WithSubscriber;

pub(super) fn hit(change: &str, id: &str) -> Value {
    json!({"_id":id,"dbnsfp":{"genename":["TP53"],"hgvsc":[change]},
        "snpeff":{"ann":[{"feature_id":"NM_012345.7","genename":"TP53","hgvs_c":change}]}})
}
// Authored expectations specify the complete public card independently of the transform.
pub(super) fn detail(change: &str, id: &str) -> Value {
    json!({"id":id,"gene":"TP53","transcript":"NM_012345.7","hgvs_c":change,
        "genome_build":"GRCh37","genome_build_provenance":"MyVariant.info provider default",
        "section_outcomes":{
            "cancerhotspots":{"outcome":"not_requested","sources":[]},
            "cbioportal":{"outcome":"not_requested","sources":[]},
            "civic":{"outcome":"not_requested","sources":[]},
            "clinvar":{"outcome":"not_requested","sources":[]},
            "gwas":{"outcome":"not_requested","sources":[]},
            "population":{"outcome":"not_requested","sources":[]},
            "predict":{"outcome":"not_requested","sources":[]}}})
}
pub(super) async fn fixture(pages: Vec<Value>) -> (TestHttpFixture, Arc<Mutex<Vec<String>>>) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    let fixture = TestHttpFixture::spawn(move |request| {
        let mut log = captured.lock().unwrap();
        let index = log.len();
        log.push(request.into());
        let (status, bytes) = match pages
            .get(index)
            .or_else(|| pages.last().filter(|page| *page == "transport"))
        {
            Some(page) if page == "transport" => return TestHttpReply::Bytes(Vec::new()),
            Some(page) if page == "failure" => {
                ("400 Bad Request", b"credential-input-canary".to_vec())
            }
            Some(page) if page == "malformed" => ("200 OK", b"{credential-input-canary".to_vec()),
            Some(page) => ("200 OK", serde_json::to_vec(page).unwrap()),
            None => ("400 Bad Request", b"unexpected-request".to_vec()),
        };
        TestHttpReply::Bytes(test_http_response(status, "application/json", &bytes))
    })
    .await;
    (fixture, requests)
}
const FIELDS: &str = "_id,cadd.phred,cadd.consequence,clinvar.gene.symbol,clinvar.rcv.accession,clinvar.rcv.version,clinvar.rcv.clinical_significance,clinvar.rcv.review_status,clinvar.rcv.conditions,clinvar.rcv.preferred_name,clinvar.rcv.last_evaluated,clinvar.rcv.number_submitters,clinvar.variant_id,clinvar.hgvs.coding,snpeff.ann.feature_id,snpeff.ann.genename,snpeff.ann.hgvs_c,snpeff.ann.hgvs_p,dbnsfp.genename,dbnsfp.hgvsp,dbnsfp.hgvsc,dbnsfp.sift.pred,dbnsfp.sift.score,dbnsfp.polyphen2.hdiv.pred,dbnsfp.revel.score,dbnsfp.revel.rankscore,dbnsfp.alphamissense.score,dbnsfp.alphamissense.pred,dbnsfp.alphamissense.rankscore,dbnsfp.clinpred.score,dbnsfp.clinpred.pred,dbnsfp.metarnn.score,dbnsfp.metarnn.pred,dbnsfp.bayesdel.add_af.score,dbnsfp.bayesdel.add_af.pred,dbnsfp.bayesdel.no_af.score,dbnsfp.bayesdel.no_af.pred,dbnsfp.phylop.100way_vertebrate.rankscore,dbnsfp.phylop.470way_mammalian.rankscore,dbnsfp.phastcons.100way_vertebrate.rankscore,dbnsfp.phastcons.470way_mammalian.rankscore,dbnsfp.gerp++.rs,dbsnp.rsid,gnomad_exome.af.af,gnomad_exome.af.af_afr,gnomad_exome.af.af_eas,gnomad_exome.af.af_nfe,gnomad_exome.af.af_sas,gnomad_exome.af.af_amr,gnomad_exome.af.af_asj,gnomad_exome.af.af_fin,gnomad_exome.af.af_afr_female,gnomad_exome.af.af_afr_male,gnomad_exome.af.af_amr_female,gnomad_exome.af.af_amr_male,gnomad_exome.af.af_eas_jpn,gnomad_exome.af.af_eas_kor,gnomad_exome.af.af_nfe_bgr,gnomad_exome.af.af_nfe_est,gnomad_exome.af.af_nfe_nwe,gnomad_exome.af.af_nfe_onf,gnomad_exome.af.af_nfe_seu,gnomad_exome.af.af_nfe_swe,gnomad_exome.af.af_oth,gnomad.exomes.af.af,gnomad.exomes.af.af_afr,gnomad.exomes.af.af_eas,gnomad.exomes.af.af_nfe,gnomad.exomes.af.af_sas,gnomad.exomes.af.af_amr,gnomad.exomes.af.af_asj,gnomad.exomes.af.af_fin,gnomad.genomes.af.af,gnomad.genomes.af.af_afr,gnomad.genomes.af.af_eas,gnomad.genomes.af.af_nfe,gnomad.genomes.af.af_sas,gnomad.genomes.af.af_amr,gnomad.genomes.af.af_asj,gnomad.genomes.af.af_fin,exac.af,exac_nontcga.af,cosmic.cosmic_id,cosmic.mut_freq,cosmic.tumor_site,cosmic.mut_nt,cgi,civic";

pub(super) fn ledger(requests: &Arc<Mutex<Vec<String>>>, change: &str, offsets: &[usize]) {
    let log = requests.lock().unwrap();
    assert_eq!(log.len(), offsets.len(), "{log:?}");
    for (request, offset) in log.iter().zip(offsets) {
        let words: Vec<_> = request.lines().next().unwrap().split_whitespace().collect();
        assert_eq!((words[0], words[2]), ("GET", "HTTP/1.1"));
        let url = reqwest::Url::parse(&format!("http://localhost{}", words[1])).unwrap();
        assert_eq!(url.path(), "/query");
        assert_eq!(
            url.query_pairs().into_owned().collect::<Vec<_>>(),
            vec![
                (
                    "q".into(),
                    format!(
                        "dbnsfp.genename:TP53 AND dbnsfp.hgvsc:\"{}\"",
                        match change {
                            "c.(19C>T)" => "c.\\(19C>T\\)",
                            "c.(19_21)C>T" => "c.\\(19_21\\)C>T",
                            "c.-7_41del" => "c.\\-7_41del",
                            "c.*19C>T" => "c.\\*19C>T",
                            "c.19+1G>A" => "c.19\\+1G>A",
                            other => other,
                        }
                    )
                ),
                ("size".into(), "50".into()),
                ("from".into(), offset.to_string()),
                ("fields".into(), FIELDS.into())
            ]
        );
        assert_eq!(request.split_once("\r\n\r\n").unwrap().1, "");
    }
}
const INVALID: &str =
    "Gene coding lookup requires an exact gene and a complete supported coding fragment.";
const LIMIT: &str = "Gene coding lookup exceeds its input limit.";
const ASSEMBLY: &str = "--assembly is only supported for genomic variant IDs";
pub(super) const AMBIGUOUS: &str = "Gene coding lookup is ambiguous. Use an exact rsID or genomic HGVS ID, or search with the gene and coding change.";
const EVIDENCE: &str = "Gene coding lookup lacks complete identity evidence. Use an exact rsID or genomic HGVS ID, or search with the gene and coding change.";
const INCOMPLETE: &str = "Gene coding lookup did not complete its candidate scan. Use an exact rsID or genomic HGVS ID, or search with the gene and coding change.";

async fn exercise(
    input: &str,
    change: &str,
    pages: Vec<Value>,
    expected: Value,
    offsets: &[usize],
    assembly: Option<variant::GenomeBuild>,
) {
    let (fixture, requests) = fixture(pages).await;
    let mut env = TestEnv::new();
    env.set("BIOMCP_MYVARIANT_BASE", &fixture.base);
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_CACHE_MODE", "off");
    let trace = tempfile::NamedTempFile::new().unwrap();
    let writer = trace.as_file().try_clone().unwrap();
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_env_filter("biomcp_cli=trace")
        .with_writer(move || writer.try_clone().unwrap())
        .finish();
    let result = async {
        if assembly.is_some() {
            variant::get_with_workflow_signals(input, &[], assembly)
                .await
                .map(|(card, _)| card)
        } else {
            variant::get(input, &[]).await
        }
    }
    .with_subscriber(subscriber)
    .await;
    let logs = std::fs::read_to_string(trace.path()).unwrap();
    assert!(!logs.contains("credential-input-canary"));
    if expected.is_object() {
        assert_eq!(
            serde_json::to_value(result.unwrap()).unwrap(),
            expected,
            "{input}"
        );
    } else {
        let error = result.unwrap_err();
        match expected.as_str().unwrap() {
            "absent" => match error {
                crate::error::BioMcpError::NotFound {
                    entity,
                    id,
                    suggestion,
                } => {
                    assert_eq!((entity.as_str(), id.as_str()), ("variant", input.trim()));
                    assert_eq!(
                        suggestion,
                        format!("Try searching: biomcp search variant \"TP53 {change}\"")
                    );
                }
                error => panic!("{error:?}"),
            },
            "failure" | "malformed" | "transport" => {
                let human = error.to_string();
                let debug = format!("{error:?}");
                let structured = crate::render::json::to_error_json(&error).unwrap();
                for observation in [&human, &debug, &structured] {
                    assert!(!observation.contains("credential-input-canary"));
                    assert!(!observation.contains("c.0019C>T"));
                }
                assert!(human.contains("MyVariant.info"));
                let payload = serde_json::from_str::<Value>(&structured).unwrap();
                assert_eq!(
                    payload["error"]["code"],
                    match expected.as_str().unwrap() {
                        "failure" => "api",
                        "malformed" => "api_json",
                        _ => "http_middleware",
                    }
                );
            }
            "legacy-invalid" => assert!(matches!(
                error,
                crate::error::BioMcpError::InvalidArgument(_)
            )),
            message => match error {
                crate::error::BioMcpError::InvalidArgument(actual) => {
                    assert_eq!(actual, message, "{input}")
                }
                error => panic!("{error:?}"),
            },
        }
    }
    ledger(&requests, change, offsets);
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn gene_coding_get_preserves_written_identity_and_complete_source_card() {
    let id = "chr17:g.101C>T";
    // A1 is deliberately first: successful ordinary callable get supplied the baseline red.
    for change in [
        "c.215C>G",
        "c.19del",
        "c.19_21dup",
        "c.19_20insAC",
        "c.19_21delinsAC",
        "c.19_21inv",
        "c.19=",
        "c.(19C>T)",
        "c.-7_41del",
        "c.*19C>T",
        "c.19+1G>A",
        "c.(19_21)C>T",
        "c.2147483648C>T",
    ] {
        exercise(
            &format!("TP53 {change}"),
            change,
            vec![json!({"total":1,"hits":[hit(change, id)]})],
            detail(change, id),
            &[0],
            None,
        )
        .await;
    }
    exercise(
        "  TP53 \t c.215C>G \n",
        "c.215C>G",
        vec![json!({"total":1,"hits":[hit("c.215C>G", id)]})],
        detail("c.215C>G", id),
        &[0],
        None,
    )
    .await;
    let padded = hit("c.0019C>T", id);
    let card = detail("c.0019C>T", id);
    let ordinary = hit("c.19C>T", "chr17:g.100C>T");
    exercise(
        "TP53 c.0019C>T",
        "c.0019C>T",
        vec![json!({"total":2,"hits":[ordinary.clone(),padded.clone()]})],
        card.clone(),
        &[0],
        None,
    )
    .await;
    exercise(
        "TP53 c.0019C>T",
        "c.0019C>T",
        vec![json!({"total":1,"hits":[ordinary]})],
        json!("absent"),
        &[0],
        None,
    )
    .await;
    let mut later_alias = padded.clone();
    later_alias["dbnsfp"]["hgvsc"] = json!(["c.19C>T", "c.0019C>T"]);
    exercise(
        "TP53 c.0019C>T",
        "c.0019C>T",
        vec![json!({"total":1,"hits":[later_alias]})],
        card.clone(),
        &[0],
        None,
    )
    .await;
    let mut displayed = padded.clone();
    displayed["snpeff"]["ann"][0] = json!({"feature_id":"NM_012346.8","genename":"TP53","hgvs_c":"c.29del","hgvs_p":"p.Ala10del"});
    let mut display_card = detail("c.29del", id);
    display_card["transcript"] = json!("NM_012346.8");
    display_card["hgvs_p"] = json!("p.Ala10del");
    exercise(
        "TP53 c.0019C>T",
        "c.0019C>T",
        vec![json!({"total":1,"hits":[displayed]})],
        display_card,
        &[0],
        None,
    )
    .await;
    let mut prefixes = padded.clone();
    prefixes["dbnsfp"]["hgvsc"] = json!(["NM_012345.7:c.0019C>T", "NM_012346.8:c.0019C>T"]);
    exercise(
        "TP53 c.0019C>T",
        "c.0019C>T",
        vec![json!({"total":1,"hits":[prefixes]})],
        card,
        &[0],
        None,
    )
    .await;
    gene_coding_get_refuses_unadmitted_inputs_before_contact().await;
    gene_coding_get_resolves_only_complete_bounded_source_identity().await;
}

async fn gene_coding_get_refuses_unadmitted_inputs_before_contact() {
    for input in [
        "TP53 c.0C>T",
        "TP53 c.19c>t",
        "TP53 c.19C>T tail",
        "TP53 c.(19C>T",
        "TP53 c.(19del)",
        "TP53 c.[19C>T;21del]",
        "tp53 c.19C>T",
        "TP53 NM_012345.7:c.19C>T",
    ] {
        exercise(input, "", vec![], json!(INVALID), &[], None).await;
    }
    for input in [
        "c.19C>T",
        "TP53 n.19C>T",
        "TP53 r.19c>u",
        "TP53 g.19C>T",
        "TP53 m.19C>T",
    ] {
        exercise(input, "", vec![], json!("legacy-invalid"), &[], None).await;
    }
    let boundary = format!("TP53 c.{}19C>T", "0".repeat(500));
    assert_eq!(boundary.len(), 512);
    let change = boundary.strip_prefix("TP53 ").unwrap();
    let mut boundary_hit = hit(change, "chr17:g.101C>T");
    boundary_hit["snpeff"]["ann"][0]["hgvs_c"] = json!("c.29del");
    exercise(
        &boundary,
        change,
        vec![json!({"total":1,"hits":[boundary_hit]})],
        detail("c.29del", "chr17:g.101C>T"),
        &[0],
        None,
    )
    .await;
    exercise(&format!(" {boundary}"), "", vec![], json!(LIMIT), &[], None).await;
    exercise(
        "TP53 c.19C>T",
        "",
        vec![],
        json!(ASSEMBLY),
        &[],
        Some(variant::GenomeBuild::Grch38),
    )
    .await;
}

async fn gene_coding_get_resolves_only_complete_bounded_source_identity() {
    let padded = hit("c.0019C>T", "chr17:g.101C>T");
    let card = detail("c.0019C>T", "chr17:g.101C>T");
    let mut wrong = padded.clone();
    wrong["dbnsfp"]["hgvsc"] = json!(["c.19C>T"]);
    let mut other = padded.clone();
    other["_id"] = json!("chr17:g.102C>T");
    other["clinvar"] = json!({"variant_id":42});
    for (pages, expected, offsets) in [
        (
            vec![
                json!({"total":51,"hits":vec![wrong.clone();50]}),
                json!({"total":51,"hits":[padded.clone()]}),
            ],
            card.clone(),
            vec![0, 50],
        ),
        (
            vec![json!({"hits":[padded.clone()]}), json!({"hits":[]})],
            card.clone(),
            vec![0, 1],
        ),
        (
            vec![json!({"total":2,"hits":[padded.clone(),other]})],
            json!(AMBIGUOUS),
            vec![0],
        ),
        (
            vec![json!({"total":1,"hits":[wrong.clone()]})],
            json!("absent"),
            vec![0],
        ),
        (vec![json!({"total":0,"hits":[]})], json!("absent"), vec![0]),
        (
            vec![
                json!({"total":2,"hits":[padded.clone()]}),
                json!({"hits":[]}),
            ],
            json!(INCOMPLETE),
            vec![0, 1],
        ),
        (
            vec![
                json!({"total":3,"hits":[padded.clone()]}),
                json!({"total":1,"hits":[]}),
            ],
            json!(INCOMPLETE),
            vec![0, 1],
        ),
        (
            vec![
                json!({"total":3,"hits":[padded.clone()]}),
                json!({"total":1,"hits":[padded.clone()]}),
                json!({"hits":[]}),
            ],
            json!(INCOMPLETE),
            vec![0, 1, 2],
        ),
        (
            vec![json!({"total":2,"hits":[padded.clone()]}), json!("failure")],
            json!("failure"),
            vec![0, 1],
        ),
        (
            vec![
                json!({"total":2,"hits":[padded.clone()]}),
                json!("malformed"),
            ],
            json!("malformed"),
            vec![0, 1],
        ),
    ] {
        exercise(
            "TP53 c.0019C>T",
            "c.0019C>T",
            pages,
            expected,
            &offsets,
            None,
        )
        .await;
    }
    let mut asserted = padded.clone();
    asserted["dbnsfp"]["hgvsc"] = json!(["c.0019C>T", "NM_012345.7:c.29del"]);
    asserted["dbnsfp"]["hgvsp"] = json!(["p.Ala10del", "p.Gly11dup"]);
    let mut reordered = asserted.clone();
    reordered["dbnsfp"]["hgvsc"] = json!(["NM_012345.7:c.29del", "c.0019C>T"]);
    reordered["dbnsfp"]["hgvsp"] = json!(["p.Gly11dup", "p.Ala10del"]);
    exercise(
        "TP53 c.0019C>T",
        "c.0019C>T",
        vec![json!({"total":2,"hits":[asserted,reordered]})],
        card.clone(),
        &[0],
        None,
    )
    .await;
    for (field, value, expected) in [
        ("_id", json!(""), EVIDENCE),
        ("_id", json!("chr17:g.101c>t"), AMBIGUOUS),
        ("genename", Value::Null, EVIDENCE),
        ("genename", json!(["TP53", "OTHER"]), EVIDENCE),
        ("hgvsc", Value::Null, EVIDENCE),
        ("hgvsc", json!(["NM_012345.7:c.0019C>T"]), AMBIGUOUS),
        ("hgvsc", json!(["c.0019C>T", "c.0019C>T"]), AMBIGUOUS),
        ("hgvsp", json!(["p.Ala10del"]), AMBIGUOUS),
    ] {
        let mut neighbor = padded.clone();
        if field == "_id" {
            neighbor[field] = value;
        } else {
            neighbor["dbnsfp"][field] = value;
        }
        exercise(
            "TP53 c.0019C>T",
            "c.0019C>T",
            vec![json!({"total":2,"hits":[padded.clone(),neighbor]})],
            json!(expected),
            &[0],
            None,
        )
        .await;
    }
    exercise(
        "TP53 c.0019C>T",
        "c.0019C>T",
        vec![
            json!({"total":2,"hits":[padded.clone()]}),
            json!("transport"),
        ],
        json!("transport"),
        &[0, 1, 1, 1, 1],
        None,
    )
    .await;
    // The positive control and nearest neighbors differ only in total or the received tail.
    for (total, tail, expected) in [
        (1000, false, card.clone()),
        (1001, false, json!(INCOMPLETE)),
        (1000, true, json!(INCOMPLETE)),
    ] {
        let mut pages = vec![json!({"total":total,"hits":vec![padded.clone();50]}); 20];
        if tail {
            pages[19]["hits"] = json!(vec![padded.clone(); 51]);
        }
        exercise(
            "TP53 c.0019C>T",
            "c.0019C>T",
            pages,
            expected,
            &(0..20).map(|n| n * 50).collect::<Vec<_>>(),
            None,
        )
        .await;
    }
    exercise(
        "TP53 c.0019C>T",
        "c.0019C>T",
        vec![json!({"hits":vec![padded;50]}); 20],
        json!(INCOMPLETE),
        &(0..20).map(|n| n * 50).collect::<Vec<_>>(),
        None,
    )
    .await;
}
