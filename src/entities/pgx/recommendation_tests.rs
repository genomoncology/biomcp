use super::*;
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn tpmt_rows() -> Vec<CpicRecommendationRow> {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/sources/cpic/recommendation_tpmt_20260901.json"
    )))
    .expect("TPMT recommendation fixture")
}

#[test]
fn recommendations_keep_complete_deterministic_genotypes() {
    let source_rows = tpmt_rows();
    let target_recommendation = source_rows
        .iter()
        .find(|row| row.recommendationid == Some(8_480_061))
        .and_then(|row| row.drugrecommendation.as_deref())
        .expect("target source recommendation");
    let recommendations = map_recommendations(&source_rows, Some("TPMT"));
    let values = serde_json::to_value(recommendations).expect("recommendation JSON");
    let rows = values.as_array().expect("recommendation array");
    let alternative = rows
        .iter()
        .find(|row| row["recommendation"].as_str() == Some(target_recommendation))
        .expect("mapped target recommendation");

    assert_eq!(
        alternative["genotype"],
        json!([
            ["TPMT", "Normal Metabolizer"],
            ["NUDT15", "Poor Metabolizer"]
        ])
    );
    assert_eq!(
        alternative["activity_score"],
        json!([["TPMT", "n/a"], ["NUDT15", "n/a"]])
    );
    assert_eq!(
        alternative["implication"],
        json!([
            ["TPMT", "Normal thiopurine metabolism"],
            ["NUDT15", "Greatly increased risk of toxicity"]
        ])
    );

    let mut keys = std::collections::HashMap::<(String, String), String>::new();
    for row in rows {
        let key = (
            row["drugname"].as_str().expect("drug name").to_owned(),
            row["genotype"].to_string(),
        );
        let recommendation = row["recommendation"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        if let Some(previous) = keys.insert(key, recommendation.clone()) {
            assert_eq!(
                previous, recommendation,
                "one genotype cannot imply conflicting advice"
            );
        }
    }
}

#[test]
fn recommendations_render_every_available_gene_without_hashmap_fallbacks() {
    let rows = tpmt_rows();
    let values = serde_json::to_value(map_recommendations(&rows[2..], Some("TPMT")))
        .expect("recommendation JSON");
    assert_eq!(
        values[0]["genotype"],
        json!([
            ["CYP2C9", "Intermediate Metabolizer"],
            ["VKORC1", "Increased Sensitivity"]
        ])
    );

    let single_gene: Vec<CpicRecommendationRow> = serde_json::from_value(json!([{
        "drugname": "codeine",
        "phenotypes": {"CYP2D6": "Poor Metabolizer"}
    }]))
    .expect("single-gene recommendation");
    let single = serde_json::to_value(map_recommendations(&single_gene, Some("CYP2D6")))
        .expect("single-gene JSON");
    assert_eq!(
        single[0]["genotype"],
        json!([["CYP2D6", "Poor Metabolizer"]])
    );
}

#[test]
fn recommendation_mapping_does_not_override_the_section_limit() {
    let seed = tpmt_rows().remove(0);
    let source_rows = vec![seed; 50];
    assert_eq!(map_recommendations(&source_rows, Some("TPMT")).len(), 50);
}

#[tokio::test]
async fn recommendations_fetch_bounded_gene_drug_coverage() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind CPIC fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let recommendations = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/sources/cpic/recommendation_cyp2d6_20260803.json"
    ));
    let coverage =
        r#"[{"drugname":"codeine"},{"drugname":"amitriptyline"},{"drugname":"codeine"}]"#;
    let server = tokio::spawn(async move {
        let mut requests = Vec::new();
        for body in [recommendations, coverage] {
            let Ok(accepted) =
                tokio::time::timeout(Duration::from_millis(500), listener.accept()).await
            else {
                break;
            };
            let (mut stream, _) = accepted.expect("accept CPIC request");
            let mut request = Vec::new();
            loop {
                let mut chunk = [0_u8; 4096];
                let read = stream.read(&mut chunk).await.expect("read CPIC request");
                request.extend_from_slice(&chunk[..read]);
                if read == 0 || request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                    break;
                }
            }
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Range: 0-1/2\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .await
                .expect("write CPIC response");
            requests.push(String::from_utf8(request).expect("request text"));
        }
        requests
    });
    let client =
        CpicClient::with_test_client(crate::sources::test_client().expect("test client"), base);
    let result = get_with_cpic(
        "CYP2D6",
        &PgxGetOptions {
            sections: vec!["recommendations".into()],
            limit: 10,
            offset: 0,
            full: false,
        },
        &client,
    )
    .await
    .expect("focused recommendations");

    assert!(result.interactions.is_empty());
    assert!(!result.recommendations.is_empty());
    let requests = server.await.expect("CPIC fixture server");
    assert_eq!(
        requests.len(),
        2,
        "recommendations need an explicit drug coverage request"
    );
    assert!(requests[0].contains("limit=11") && requests[0].contains("offset=0"));
    assert!(requests[1].starts_with("GET /recommendation_view?"));
    assert!(requests[1].contains("lookupkey-%3E%3ECYP2D6=not.is.null"));
    assert!(requests[1].contains("select=drugname"));
    assert!(requests[1].contains("limit=200"));
    assert!(!requests.iter().any(|request| request.contains("pair_view")));

    let json = serde_json::to_value(result).expect("PGx JSON");
    assert_eq!(
        json["recommendation_drugs"],
        serde_json::json!(["amitriptyline", "codeine"]),
        "coverage drugs must be unique and deterministic"
    );
}
