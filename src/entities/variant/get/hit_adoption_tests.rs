//! Cached opaque readers through the detail response and Markdown.
use crate::entities::article::test_support::{TestEnv, TestHttpFixture, TestHttpReply, test_http_response};
use crate::entities::variant;
use serde_json::json;
use std::sync::{Arc, Mutex};

#[tokio::test]
#[serial_test::serial(source_env)]
async fn cached_opaque_detail_keeps_fallbacks_caps_and_requested_sections() {
    for (count, sections) in [(1, vec![]), (1, vec!["cgi"]), (1, vec!["civic"]), (21, vec!["all"]), (0, vec![])] {
        let cgi = json!([false,{"drug":"  "},{"drug":[""," DrugA "],"association":{"x":" Responsive "},"primary_tumor_type":null,"tumor_type":"fallback","evidence_level":"A","source":" S "},{"drug":"DrugB","tumor_type":" fallback ","evidence":" B "}]);
        let mut rows = cgi.as_array().unwrap().clone();
        if count == 21 { rows.extend((0..11).map(|i| json!({"drug":format!("Drug{i}")}))); }
        let evidence = json!({"evidenceType":"Predictive","disease":{"displayName":null,"name":"fallback"},"therapies":[{"name":" A "},{"name":" "}],"source":{"sourceType":"PUBMED","citation":"PMID:111"}});
        let body = json!({"_id":"chr1:g.101A>T","dbnsfp":{"genename":"GENE"},"cgi":rows,"civic":{"molecularProfiles":[{"name":" "},{"name":" Profile ","evidenceItems":vec![evidence;count]}]}});
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = requests.clone();
        let fixture = TestHttpFixture::spawn(move |request| {
            captured.lock().unwrap().push(request.to_owned());
            if request.starts_with("GET /variant/") {
                TestHttpReply::Bytes(test_http_response("200 OK", "application/json", &serde_json::to_vec(&body).unwrap()))
            } else { TestHttpReply::Bytes(test_http_response("400 Bad Request", "application/json", b"{}")) }
        }).await;
        let mut env = TestEnv::new();
        for name in ["BIOMCP_MYVARIANT_BASE", "BIOMCP_GNOMAD_BASE", "BIOMCP_CBIOPORTAL_BASE", "BIOMCP_CIVIC_BASE"] { env.set(name, &fixture.base); }
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
        env.set("BIOMCP_CACHE_MODE", "off");
        let sections = sections.into_iter().map(str::to_owned).collect::<Vec<_>>();
        let result = variant::get("chr1:g.101A>T", &sections).await.unwrap();
        let markdown = crate::render::markdown::variant_markdown(&result, &sections).unwrap();
        let actual = serde_json::to_value(&result).unwrap();
        let cgi_requested = sections.iter().any(|s| s == "cgi" || s == "all");
        if cgi_requested {
            assert_eq!(actual["cgi_associations"][0], json!({"drug":"DrugA","association":"Responsive","evidence_level":"A","source":"S"}));
            assert_eq!(actual["cgi_associations"][1], json!({"drug":"DrugB","tumor_type":"fallback","evidence_level":"B"}));
            assert_eq!(actual["cgi_associations"].as_array().unwrap().len(), if count == 21 {10} else {2});
        } else { assert!(actual.get("cgi_associations").is_none()); }
        if count == 0 { assert!(actual.get("civic").is_none()); }
        else {
            let items = actual["civic"]["cached_evidence"].as_array().unwrap();
            assert_eq!(items.len(), count.min(20));
            assert_eq!(items[0], json!({"id":0,"name":"cached","molecular_profile":"Profile","evidence_type":"Predictive","evidence_level":"-","significance":"-","status":"-","therapies":["A"]}));
            assert!(actual["civic"].get("graphql").is_none());
            if sections.is_empty() { assert!(markdown.contains("predictive")); assert!(!markdown.contains("| cached |")); }
        }
        // This source has no protein identity, so requested live CIViC remains inapplicable.
        assert!(!requests.lock().unwrap().iter().any(|r| r.contains("graphql") || r.contains("cosmic")));
    }
}
