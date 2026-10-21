use super::*;

const FIRST_PAGE: &str =
    include_str!("../../../../testdata/replay/responses/mydisease/melanoma-page-one.json");
const SECOND_PAGE: &str =
    include_str!("../../../../testdata/replay/responses/mydisease/melanoma-page-two.json");
const DETAIL: &str = r#"{"_id":"MONDO:0005105","disease_ontology":{"name":"melanoma","synonyms":{"exact":["malignant melanoma"]}}}"#;

#[tokio::test]
async fn melanoma_resolves_only_after_both_recorded_pages_are_complete() {
    let (client, requests, mut server) = exact_resolution_fixture(|request| {
        if request.starts_with("GET /disease/") {
            DETAIL
        } else if request.contains("from=50") {
            SECOND_PAGE
        } else {
            FIRST_PAGE
        }
    })
    .await;
    let terms = resolve_exact_disease_terms(&client, "melanoma")
        .await
        .expect("complete recorded melanoma search");
    server.finish().unwrap();
    assert_eq!(terms.canonical_id.as_deref(), Some("MONDO:0005105"));
    assert_eq!(terms.synonyms, vec!["malignant melanoma"]);
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 3);
    assert!(requests[1].contains("size=50&from=50"));
    assert!(requests[2].starts_with("GET /disease/MONDO:0005105?"));
}

#[tokio::test]
async fn a_second_exact_holder_on_the_later_page_prevents_resolution() {
    let (client, requests, mut server) = exact_resolution_fixture_owned(|request| {
        if !request.contains("from=50") {
            return FIRST_PAGE.to_owned();
        }
        let mut page: serde_json::Value = serde_json::from_str(SECOND_PAGE).unwrap();
        page["hits"][0]["disease_ontology"]["name"] = "melanoma".into();
        page.to_string()
    })
    .await;
    let terms = resolve_exact_disease_terms(&client, "melanoma")
        .await
        .expect("ambiguity retains the literal term");
    server.finish().unwrap();
    assert!(terms.canonical_id.is_none());
    assert!(terms.synonyms.is_empty());
    assert_eq!(requests.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn incomplete_or_unstable_pages_never_fetch_selected_detail() {
    for fault in ["over_bound", "changed_total", "short_page", "duplicate"] {
        let (client, requests, mut server) = exact_resolution_fixture_owned(move |request| {
            let later = request.contains("from=50");
            let mut page: serde_json::Value =
                serde_json::from_str(if later { SECOND_PAGE } else { FIRST_PAGE }).unwrap();
            match (fault, later) {
                ("over_bound", _) => page["total"] = 201.into(),
                ("changed_total", true) => page["total"] = 89.into(),
                ("short_page", true) => {
                    page["hits"].as_array_mut().unwrap().pop();
                }
                ("duplicate", true) => page["hits"][0]["_id"] = "MONDO:0005105".into(),
                _ => {}
            }
            page.to_string()
        })
        .await;
        let error = resolve_exact_disease_terms(&client, "melanoma")
            .await
            .expect_err(fault);
        server.finish().unwrap();
        assert!(
            matches!(error, BioMcpError::SourceUnavailable { source_name, .. } if source_name == "MyDisease.info")
        );
        assert_eq!(
            requests.lock().unwrap().len(),
            if fault == "over_bound" { 1 } else { 2 }
        );
    }
}

#[tokio::test]
async fn exact_resolution_checks_all_four_pages_before_fetching_detail() {
    let (client, requests, mut server) = exact_resolution_fixture_owned(|request| {
        if request.starts_with("GET /disease/") { return DETAIL.to_owned(); }
        let offset = [50, 100, 150].into_iter().find(|offset| request.contains(&format!("from={offset}&"))).unwrap_or(0);
        let hits: Vec<_> = (offset..(offset + 50).min(190)).map(|index| {
            if index == 0 { serde_json::json!({"_id":"MONDO:0005105","disease_ontology":{"name":"melanoma"}}) }
            else { serde_json::json!({"_id":format!("MONDO:{}", 1000000 + index),"disease_ontology":{"name":format!("unrelated fixture {index}")}}) }
        }).collect();
        serde_json::json!({"total":190,"hits":hits}).to_string()
    }).await;
    let terms = resolve_exact_disease_terms(&client, "melanoma")
        .await
        .expect("four complete pages within the bound");
    server.finish().unwrap();
    assert_eq!(terms.canonical_id.as_deref(), Some("MONDO:0005105"));
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 5);
    assert!(requests[3].contains("from=150"));
    assert!(requests[4].starts_with("GET /disease/"));
}

#[tokio::test]
async fn exact_resolution_deadline_cancels_a_provider_that_never_responds() {
    use tokio::io::AsyncReadExt;
    let (received_tx, mut received_rx) = tokio::sync::mpsc::unbounded_channel();
    let (closed_tx, mut closed_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut server = crate::test_support::TestHttpServer::spawn(move |mut stream| {
        let received_tx = received_tx.clone();
        let closed_tx = closed_tx.clone();
        async move {
            let Some(request) = crate::test_support::read_request(&mut stream).await? else {
                return Ok(());
            };
            received_tx
                .send(request.len())
                .map_err(std::io::Error::other)?;
            // EOF is the cancellation event; the fixture never sends a response.
            let mut remaining = Vec::new();
            stream.read_to_end(&mut remaining).await?;
            closed_tx.send(()).map_err(std::io::Error::other)?;
            Ok(())
        }
    })
    .expect("start stalled disease-resolution fixture");
    let client = MyDiseaseClient::new_for_test(server.base.clone()).unwrap();
    let started = tokio::time::Instant::now();
    let mut task =
        tokio::spawn(async move { resolve_exact_disease_terms(&client, "melanoma").await });
    let received = tokio::select! {
        result = &mut task => panic!("resolution ended before provider request: {result:?}"),
        received = received_rx.recv() => received.expect("provider request"),
    };
    assert!(
        received > 0,
        "provider must receive the request before cancellation"
    );
    let error = task
        .await
        .unwrap()
        .expect_err("provider work must end at the resolution deadline");
    assert!(
        matches!(error, BioMcpError::SourceUnavailable { source_name, .. } if source_name == "MyDisease.info")
    );
    assert!(started.elapsed() >= std::time::Duration::from_secs(15));
    // Dropping the timed-out request must also close the provider connection.
    closed_rx
        .recv()
        .await
        .expect("provider observes cancellation EOF");
    server.finish().unwrap();
}
