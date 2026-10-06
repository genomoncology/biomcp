//! Original invented MIT provider facts; complete caller expectations.
use crate::entities::article::test_support::TestEnv;
use crate::entities::article::test_support::{TestHttpFixture, TestHttpReply, test_http_response};
use crate::entities::variant;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

pub(super) fn hit(change: &str) -> Value {
    json!({"_id":"chr17:g.101C>T","snpeff":{"ann":[{"feature_id":"NM_012345.7","genename":"TP53","hgvs_c":change,"hgvs_p":"p.Gly6del"}]}})
}
pub(super) fn card(change: &str) -> Value {
    let mut value = json!({"id":"chr17:g.101C>T","gene":"TP53","transcript":"NM_012345.7","hgvs_c":change,
        "genome_build":"GRCh37","genome_build_provenance":"MyVariant.info provider default",
        "section_outcomes":{
            "cancerhotspots":{"outcome":"not_requested","sources":[]},
            "cbioportal":{"outcome":"not_requested","sources":[]},
            "civic":{"outcome":"not_requested","sources":[]},
            "clinvar":{"outcome":"not_requested","sources":[]},
            "gwas":{"outcome":"not_requested","sources":[]},
            "population":{"outcome":"not_requested","sources":[]},
            "predict":{"outcome":"not_requested","sources":[]}}});
    value["hgvs_p"] = json!("p.Gly6del");
    value
}
pub(super) async fn fixture(pages: Vec<Value>) -> (TestHttpFixture, Arc<Mutex<Vec<String>>>) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    let fixture = TestHttpFixture::spawn(move |request| {
        let mut log = captured.lock().unwrap();
        let index = log.len();
        log.push(request.into());
        let bytes = match pages.get(index) {
            Some(page) if page == "malformed" => b"{credential-input-canary".to_vec(),
            Some(page) => serde_json::to_vec(page).unwrap(),
            None => b"{unexpected-contact-canary".to_vec(),
        };
        TestHttpReply::Bytes(test_http_response("200 OK", "application/json", &bytes))
    })
    .await;
    (fixture, requests)
}

const FIELDS: &str = "_id,cadd.phred,cadd.consequence,clinvar.gene.symbol,clinvar.rcv.accession,clinvar.rcv.version,clinvar.rcv.clinical_significance,clinvar.rcv.review_status,clinvar.rcv.conditions,clinvar.rcv.preferred_name,clinvar.rcv.last_evaluated,clinvar.rcv.number_submitters,clinvar.variant_id,snpeff.ann.feature_id,snpeff.ann.genename,snpeff.ann.hgvs_c,snpeff.ann.hgvs_p,dbnsfp.genename,dbnsfp.hgvsp,dbnsfp.hgvsc,dbnsfp.sift.pred,dbnsfp.sift.score,dbnsfp.polyphen2.hdiv.pred,dbnsfp.revel.score,dbnsfp.revel.rankscore,dbnsfp.alphamissense.score,dbnsfp.alphamissense.pred,dbnsfp.alphamissense.rankscore,dbnsfp.clinpred.score,dbnsfp.clinpred.pred,dbnsfp.metarnn.score,dbnsfp.metarnn.pred,dbnsfp.bayesdel.add_af.score,dbnsfp.bayesdel.add_af.pred,dbnsfp.bayesdel.no_af.score,dbnsfp.bayesdel.no_af.pred,dbnsfp.phylop.100way_vertebrate.rankscore,dbnsfp.phylop.470way_mammalian.rankscore,dbnsfp.phastcons.100way_vertebrate.rankscore,dbnsfp.phastcons.470way_mammalian.rankscore,dbnsfp.gerp++.rs,dbsnp.rsid,gnomad_exome.af.af,gnomad_exome.af.af_afr,gnomad_exome.af.af_eas,gnomad_exome.af.af_nfe,gnomad_exome.af.af_sas,gnomad_exome.af.af_amr,gnomad_exome.af.af_asj,gnomad_exome.af.af_fin,gnomad_exome.af.af_afr_female,gnomad_exome.af.af_afr_male,gnomad_exome.af.af_amr_female,gnomad_exome.af.af_amr_male,gnomad_exome.af.af_eas_jpn,gnomad_exome.af.af_eas_kor,gnomad_exome.af.af_nfe_bgr,gnomad_exome.af.af_nfe_est,gnomad_exome.af.af_nfe_nwe,gnomad_exome.af.af_nfe_onf,gnomad_exome.af.af_nfe_seu,gnomad_exome.af.af_nfe_swe,gnomad_exome.af.af_oth,gnomad.exomes.af.af,gnomad.exomes.af.af_afr,gnomad.exomes.af.af_eas,gnomad.exomes.af.af_nfe,gnomad.exomes.af.af_sas,gnomad.exomes.af.af_amr,gnomad.exomes.af.af_asj,gnomad.exomes.af.af_fin,gnomad.genomes.af.af,gnomad.genomes.af.af_afr,gnomad.genomes.af.af_eas,gnomad.genomes.af.af_nfe,gnomad.genomes.af.af_sas,gnomad.genomes.af.af_amr,gnomad.genomes.af.af_asj,gnomad.genomes.af.af_fin,exac.af,exac_nontcga.af,cosmic.cosmic_id,cosmic.mut_freq,cosmic.tumor_site,cosmic.mut_nt,cgi,civic";

pub(super) fn ledger(
    requests: &Arc<Mutex<Vec<String>>>,
    change: &str,
    transcript: &str,
    offsets: &[usize],
) {
    let escaped = match change {
        "c.678-14_678-3del" => "c.678\\-14_678\\-3del",
        "c.-7_-6del" => "c.\\-7_\\-6del",
        "c.*7_*8del" => "c.\\*7_\\*8del",
        "c.-7_41del" => "c.\\-7_41del",
        "c.(17_19)_(31_34)del" => "c.\\(17_19\\)_\\(31_34\\)del",
        "c.-541-5533del" => "c.\\-541\\-5533del",
        "c.(19C>T)" => "c.\\(19C>T\\)",
        "c.(19_21)C>T" => "c.\\(19_21\\)C>T",
        "c.19+1G>A" => "c.19\\+1G>A",
        "c.-7dupAC" => "c.\\-7dupAC",
        "c.*19_20inv" => "c.\\*19_20inv",
        other => other,
    };
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
                    format!("clinvar.hgvs.coding:\"{transcript}\\:{escaped}\"")
                ),
                ("size".into(), "50".into()),
                ("from".into(), offset.to_string()),
                ("fields".into(), FIELDS.into())
            ]
        );
        assert_eq!(request.split_once("\r\n\r\n").unwrap().1, "");
    }
}
const INVALID: &str = "Transcript gene coding lookup requires an exact versioned transcript, gene and complete supported coding fragment.";
const LIMIT: &str = "Transcript gene coding lookup exceeds its input limit.";
pub(super) const AMBIGUOUS: &str =
    "Transcript gene coding lookup is ambiguous. Use an exact rsID or genomic HGVS ID.";
const EVIDENCE: &str = "Transcript gene coding lookup lacks complete identity evidence. Use an exact rsID or genomic HGVS ID.";
const INCOMPLETE: &str = "Transcript gene coding lookup did not complete its candidate scan. Use an exact rsID or genomic HGVS ID.";
const ARTICLES: &str = "Transcript gene coding lookup is not supported for variant articles. Use get variant for coding detail.";
const D: &str = "c.17_18del";
fn wrapper(change: &str) -> String {
    format!("NM_012345.7(TP53):{change}")
}
async fn exercise(
    input: &str,
    change: &str,
    pages: Vec<Value>,
    expected: Value,
    offsets: &[usize],
    assembly: bool,
    article: bool,
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
    use tracing::instrument::WithSubscriber;
    let result = async {
        if article {
            crate::entities::article::search_variant_articles(
                input,
                crate::entities::article::VariantArticleStrategy::default(),
                5,
                0,
            )
            .await
            .map(|_| Value::Null)
        } else {
            variant::get_with_workflow_signals(
                input,
                &[],
                assembly.then_some(variant::GenomeBuild::Grch38),
            )
            .await
            .map(|(card, _)| serde_json::to_value(card).unwrap())
        }
    }
    .with_subscriber(subscriber)
    .await;
    if expected.is_object() {
        assert_eq!(result.unwrap(), expected, "{input}");
    } else {
        let error = result.unwrap_err();
        if expected == "absent" {
            match error {
                crate::error::BioMcpError::NotFound {
                    entity,
                    id,
                    suggestion,
                } => {
                    assert_eq!(entity, "variant");
                    assert_eq!(id, input.trim());
                    assert_eq!(suggestion, "Try searching: biomcp search variant");
                }
                error => panic!("{error:?}"),
            }
        } else if expected == "source" {
            let payload: Value =
                serde_json::from_str(&crate::render::json::to_error_json(&error).unwrap()).unwrap();
            assert_eq!(payload["error"]["code"], "api_json");
            for observation in [
                error.to_string(),
                format!("{error:?}"),
                crate::render::json::to_error_json(&error).unwrap(),
            ] {
                assert!(!observation.contains("credential-input-canary"));
                assert!(!observation.contains(&fixture.base));
            }
        } else {
            match error {
                crate::error::BioMcpError::InvalidArgument(actual) => {
                    assert_eq!(actual, expected.as_str().unwrap())
                }
                error => panic!("{error:?}"),
            }
        }
    }
    let logs = std::fs::read_to_string(trace.path()).unwrap();
    assert!(!logs.contains("credential-input-canary"));
    assert!(!logs.contains(input.trim()));
    let transcript = input.trim().split_once('(').unwrap().0;
    ledger(&requests, change, transcript, offsets);
}
async fn success(change: &str, spaced: bool) {
    let input = if spaced {
        format!("NM_012345.7(TP53) {change}")
    } else {
        wrapper(change)
    };
    exercise(
        &input,
        change,
        vec![json!({"total":1,"hits":[hit(change)]})],
        card(change),
        &[0],
        false,
        false,
    )
    .await;
}
macro_rules! source_case { ($name:ident, $body:block) => {
    #[tokio::test] #[serial_test::serial(source_env)] async fn $name() $body
}; }
macro_rules! success_case {
    ($name:ident, $change:literal, $spaced:literal) => {
        #[tokio::test]
        #[serial_test::serial(source_env)]
        async fn $name() {
            success($change, $spaced).await;
        }
    };
}
success_case!(transcript_deletion_01_point, "c.17del", false);
success_case!(transcript_deletion_02_range, "c.17_18del", false);
success_case!(transcript_deletion_03_base, "c.17delA", false);
success_case!(transcript_deletion_04_sequence, "c.17_18delAC", false);
success_case!(
    transcript_deletion_05_intronic_offsets,
    "c.678-14_678-3del",
    false
);
success_case!(transcript_deletion_06_upstream, "c.-7_-6del", false);
success_case!(transcript_deletion_07_coding_end, "c.*7_*8del", false);
success_case!(transcript_deletion_08_mixed_markers, "c.-7_41del", false);
success_case!(
    transcript_deletion_09_uncertainty,
    "c.(17_19)_(31_34)del",
    false
);
success_case!(
    transcript_deletion_10_decimal_spelling,
    "c.0017_0018del",
    false
);
success_case!(
    transcript_deletion_11_large_position,
    "c.2147483648del",
    false
);
source_case!(transcript_deletion_12_spaced_wrapper, {
    coding_exercise(
        "NM_012345.7(TP53) c.19_20insAC",
        "c.19_20insAC",
        vec![rcv_hit("c.19_20insAC")],
        coding_card("c.19_20insAC"),
    )
    .await;
});

macro_rules! refusal_case {
    ($name:ident, $input:expr, $message:expr, $assembly:literal, $article:literal) => {
        #[tokio::test]
        #[serial_test::serial(source_env)]
        async fn $name() {
            exercise(
                &$input,
                D,
                vec![],
                json!($message),
                &[],
                $assembly,
                $article,
            )
            .await;
        }
    };
}
refusal_case!(
    transcript_deletion_13_missing_version,
    "NM_012345(TP53):c.17del",
    INVALID,
    false,
    false
);
refusal_case!(
    transcript_deletion_14_missing_gene,
    "NM_012345.7():c.17del",
    INVALID,
    false,
    false
);
refusal_case!(
    transcript_deletion_15_invalid_neighbor,
    wrapper("c.17_18delx"),
    INVALID,
    false,
    false
);
refusal_case!(
    transcript_deletion_16_predicted,
    wrapper("c.(17_18del)"),
    INVALID,
    false,
    false
);
source_case!(transcript_deletion_17_duplication, {
    let change = "c.17dup";
    coding_exercise(
        &wrapper(change),
        change,
        vec![coding_hit(change)],
        coding_card(change),
    )
    .await;
});
refusal_case!(
    transcript_deletion_18_compound,
    wrapper("c.[17del;19del]"),
    INVALID,
    false,
    false
);
refusal_case!(
    transcript_deletion_19_original_limit,
    format!(
        "{}{}",
        " ".repeat(513 - wrapper("c.19_20insAC").len()),
        wrapper("c.19_20insAC")
    ),
    LIMIT,
    false,
    false
);
refusal_case!(
    transcript_deletion_20_assembly,
    wrapper("c.19_20insAC"),
    "--assembly is only supported for genomic variant IDs",
    true,
    false
);
refusal_case!(
    transcript_deletion_44_colon_article,
    wrapper("c.19_20insAC"),
    ARTICLES,
    false,
    true
);
refusal_case!(
    transcript_deletion_45_spaced_article,
    "NM_012345.7(TP53) c.19_20insAC",
    ARTICLES,
    false,
    true
);

include!("transcript_deletion_source_tests.rs");

include!("transcript_coding_edit_tests.rs");
