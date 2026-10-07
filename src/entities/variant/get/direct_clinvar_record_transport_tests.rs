//! Complete shared direct records survive the actual client and shipped channels.
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use biomcp_mcp_contract_client::{ContractHarness, first_text};
use rmcp::model::CallToolRequestParams;
use serde_json::{Value, json};

const XML: &str = r#"<ClinVarResult-Set>
<VariationArchive VariationID="123" Accession="VCV123" Version="2" NumberOfSubmissions="2" NumberOfSubmitters="2">
<RecordStatus>current</RecordStatus><ClassifiedRecord>
<Classifications><GermlineClassification DateLastEvaluated="record-date"><Description>record-label</Description><ReviewStatus>record-review</ReviewStatus></GermlineClassification></Classifications>
<RCVList><RCVAccession Accession="RCV123" Version="4" RecordStatus="aggregate-status">
<ClassifiedConditionList><ClassifiedCondition>aggregate-condition</ClassifiedCondition></ClassifiedConditionList>
<RCVClassifications><GermlineClassification><ReviewStatus>aggregate-review</ReviewStatus><Description DateLastEvaluated="aggregate-date" NumberOfSubmitters="2" SubmissionCount="2">aggregate-label</Description></GermlineClassification></RCVClassifications>
</RCVAccession></RCVList>
<ClinicalAssertionList>
<ClinicalAssertion ID="1"><ClinVarAccession Accession="SCV1" Version="1" SubmitterName="submitter-one"/><RecordStatus>current</RecordStatus>
<Classification DateLastEvaluated="submission-date"><GermlineClassification>submission-label</GermlineClassification><ReviewStatus>submission-review</ReviewStatus><Comment>public-comment</Comment><Citation><ID Source="PubMed">111</ID><URL>https://example.invalid/111</URL></Citation></Classification>
<AttributeSet><Attribute Type="AssertionMethod">criteria-one</Attribute></AttributeSet>
</ClinicalAssertion>
<ClinicalAssertion ID="2" ContributesToAggregateClassification="false"><ClinVarAccession Accession="SCV2" Version="3" SubmitterName="submitter-two"/><RecordStatus>current</RecordStatus>
<Classification DateLastEvaluated="noncontributor-date"><OncogenicityClassification>noncontributor-label</OncogenicityClassification><ReviewStatus>noncontributor-review</ReviewStatus></Classification>
</ClinicalAssertion>
</ClinicalAssertionList>
<TraitMappingList><TraitMapping ClinicalAssertionID="1"><MedGen Name="submission-condition"/></TraitMapping><TraitMapping ClinicalAssertionID="2"><MedGen Name="noncontributor-condition"/></TraitMapping></TraitMappingList>
<Ignored>private-provider-marker</Ignored>
</ClassifiedRecord></VariationArchive></ClinVarResult-Set>"#;

fn expected_record() -> Value {
    json!({
        "source":"NCBI ClinVar", "variation_id":123, "accession":"VCV123", "version":2,
        "record_status":"current", "number_submissions":2, "number_submitters":2,
        "germline_classification":{"classification":"record-label","review_status":"record-review","evaluation_date":"record-date"},
        "aggregates":[{"source":"NCBI ClinVar","accession":"RCV123","version":4,
            "classification_domain":"germline","classification":"aggregate-label",
            "review_status":"aggregate-review","evaluation_date":"aggregate-date",
            "record_status":"aggregate-status","number_submitters":2,"submission_count":2,
            "conditions":["aggregate-condition"]}],
        "submissions":[
            {"source":"NCBI ClinVar","accession":"SCV1","version":1,"classification_domain":"germline",
             "classification":"submission-label","review_status":"submission-review","evaluation_date":"submission-date",
             "record_status":"current","submitter":"submitter-one","contributes_to_aggregate_classification":null,
             "conditions":["submission-condition"],"criteria":"criteria-one",
             "citations":[{"source":"PubMed","id":"111","url":"https://example.invalid/111"}],"public_comment":"public-comment"},
            {"source":"NCBI ClinVar","accession":"SCV2","version":3,"classification_domain":"oncogenicity",
             "classification":"noncontributor-label","review_status":"noncontributor-review","evaluation_date":"noncontributor-date",
             "record_status":"current","submitter":"submitter-two","contributes_to_aggregate_classification":false,
             "conditions":["noncontributor-condition"],"citations":[]}
        ]
    })
}

fn check_channel(text: &str, json_mode: bool) {
    assert!(!text.contains("private-provider-marker"), "{text}");
    if json_mode {
        let card: Value = serde_json::from_str(text).unwrap();
        assert_eq!(card["clinvar"], expected_record());
        assert_eq!(
            card["section_outcomes"]["clinvar"],
            json!({"outcome":"data","sources":["NCBI ClinVar"]})
        );
    } else {
        for literal in [
            "Source: NCBI ClinVar",
            "Variation ID: 123 (VCV123.2)",
            "Record-level germline classification: record-label; record-review; evaluated record-date",
            "VCV record status: current",
            "VCV submitters: 2",
            "RCV123.4 [germline]: aggregate-label; aggregate-review; evaluated aggregate-date; 2 submitter(s); RCV status aggregate-status",
            "SCV1.1 [germline]: submission-label; SCV status current; submitter-one; submission-review; evaluated submission-date",
            "SCV2.3 [oncogenicity]: noncontributor-label; SCV status current; submitter-two; contributes to aggregate: false; noncontributor-review; evaluated noncontributor-date",
            "Conditions: aggregate-condition.",
            "Conditions: submission-condition.",
            "Conditions: noncontributor-condition.",
            "Criteria: criteria-one.",
            "Citations: PubMed:111.",
            "Public comment: public-comment",
        ] {
            assert!(text.contains(literal), "missing {literal:?}: {text}");
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn complete_shared_direct_record_keeps_metadata_and_noncontributors_across_channels() {
    let fixture = TestHttpFixture::spawn(|request| {
        if request.starts_with("GET /efetch.fcgi") {
            TestHttpReply::Bytes(test_http_response("200 OK", "application/xml", XML.as_bytes()))
        } else {
            TestHttpReply::Bytes(test_http_response("200 OK", "application/json",
                br#"{"_id":"chr7:g.140453136A>T","dbnsfp":{"genename":"BRAF"},"clinvar":{"variant_id":123}}"#))
        }
    }).await;
    let cache = tempfile::tempdir().unwrap();
    let envs = [
        ("BIOMCP_MYVARIANT_BASE", fixture.base.clone()),
        ("BIOMCP_CLINVAR_BASE", fixture.base.clone()),
        ("BIOMCP_TEST_UNPACED_ORIGIN", fixture.base.clone()),
        ("BIOMCP_CACHE_MODE", "off".into()),
        ("BIOMCP_CACHE_DIR", cache.path().display().to_string()),
        ("RUST_LOG", "off,reqwest_retry=error".into()),
        ("ONCOKB_TOKEN", "".into()),
        ("NCBI_API_KEY", "".into()),
    ];
    let mut env = TestEnv::new();
    for (key, value) in &envs {
        env.set(key, value);
    }
    let native = super::get("chr7:g.140453136A>T", &["clinvar".into()])
        .await
        .unwrap();
    // The actual section stores the shared record; a local mirror cannot satisfy this caller.
    let record: &biodata::ClinVarRecordProjection = native.clinvar.as_ref().unwrap();
    assert_eq!(
        serde_json::to_value(record.as_record_target()).unwrap(),
        expected_record()
    );
    let harness = ContractHarness::new(
        std::env::var_os("BIOMCP_BIN").unwrap(),
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")),
    );
    let client = harness.spawn_stdio_client(&envs).await.unwrap();
    for json_mode in [true, false] {
        let mut args = vec!["get", "variant", "chr7:g.140453136A>T", "clinvar"];
        if json_mode {
            args.push("--json");
        }
        let output = tokio::process::Command::new(&harness.biomcp_bin)
            .args(args)
            .envs(envs.iter().cloned())
            .output()
            .await
            .unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty(), "{:?}", output.stderr);
        check_channel(std::str::from_utf8(&output.stdout).unwrap(), json_mode);
        for tool in ["get", "biomcp"] {
            let arguments = if tool == "get" {
                json!({"entity":"variant","id":"chr7:g.140453136A>T","sections":["clinvar"],"json":json_mode})
            } else {
                json!({"command":"biomcp get variant 'chr7:g.140453136A>T' clinvar","json":json_mode})
            };
            let result = client
                .peer()
                .call_tool(
                    CallToolRequestParams::new(tool)
                        .with_arguments(arguments.as_object().unwrap().clone()),
                )
                .await
                .unwrap();
            assert!(!result.is_error.unwrap_or_default());
            check_channel(first_text(&result.content), json_mode);
        }
    }
    client.cancel().await.unwrap();
}
