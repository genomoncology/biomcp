//! Tier 3 — the PharmacoDB size guard. Pure: synthetic bodies at and above the
//! measured worst cases. No network, no server.
//!
//! The bodies here are generated rather than recorded, because the two measured
//! worst cases are 19.4 MB and 18.6 MB and neither belongs in the repository.

use super::super::*;
use reqwest::StatusCode;

const URL: &str = "https://pharmacodb.ca/graphql";
/// The K-562 counts case: 78,373 rows, the largest row count measured.
const K562_ROW_COUNT: usize = 78_373;

fn synthetic_counts_body(rows: usize) -> Vec<u8> {
    let mut body = String::from(r#"{"data":{"experiments":["#);
    for index in 0..rows {
        if index > 0 {
            body.push(',');
        }
        let dataset = PHARMACODB_DATASET_NAMES[index % PHARMACODB_DATASET_NAMES.len()];
        body.push_str(&format!(
            r#"{{"id":{},"dataset":{{"name":"{dataset}"}}}}"#,
            4_000_000 + index
        ));
    }
    body.push_str("]}}");
    body.into_bytes()
}

fn synthetic_row_body(rows: usize) -> Vec<u8> {
    let mut body = String::from(r#"{"data":{"experiments":["#);
    for index in 0..rows {
        if index > 0 {
            body.push(',');
        }
        let dataset = PHARMACODB_DATASET_NAMES[index % PHARMACODB_DATASET_NAMES.len()];
        body.push_str(&format!(
            r#"{{"id":{id},"cell_line":{{"id":905,"uid":"K562_631_2019","name":"K-562"}},"compound":{{"id":{id},"uid":"PDBC0{id}","name":"synthetic compound {id}"}},"dataset":{{"name":"{dataset}"}},"tissue":{{"name":"Myeloid"}},"profile":{{"AAC":0.12345678,"IC50":null,"EC50":1.23456789,"Einf":100,"HS":2.17297756,"DSS1":null}}}}"#,
            id = 5_000_000 + index
        ));
    }
    body.push_str("]}}");
    body.into_bytes()
}

fn response(body: Vec<u8>) -> reqwest::Response {
    http::Response::builder()
        .status(StatusCode::OK)
        .body(reqwest::Body::from(body))
        .expect("synthetic response")
        .into()
}

// Ticket 1205 acceptance 6: the measured worst cases parse, and a body above the
// cap is refused.

#[test]
fn the_k562_counts_case_parses_and_sums_to_every_row() {
    let counts = PharmacoDbClient::decode_experiment_counts(
        URL,
        StatusCode::OK,
        &synthetic_counts_body(K562_ROW_COUNT),
    )
    .unwrap();

    assert_eq!(
        counts.iter().map(|row| row.count).sum::<usize>(),
        K562_ROW_COUNT
    );
    assert_eq!(counts.len(), PHARMACODB_DATASET_NAMES.len());
}

#[test]
fn the_largest_measured_row_body_parses_and_returns_its_rows() {
    // 19.4 MB is the measured K-562 full-field body. This one is built past it.
    let body = synthetic_row_body(63_000);
    assert!(
        body.len() > 19_400_000,
        "the synthetic body must reach the measured worst case, found {} bytes",
        body.len()
    );
    assert!(
        body.len() < PHARMACODB_MAX_BODY_BYTES,
        "and stay under the cap"
    );

    let rows = PharmacoDbClient::decode_experiments(URL, StatusCode::OK, &body).unwrap();

    assert_eq!(rows.len(), 63_000);
    assert_eq!(rows[0].cell_line_name, "K-562");
    assert!(rows[0].ic50.is_none());
}

#[tokio::test]
async fn a_body_over_the_cap_is_refused_while_it_streams() {
    let error =
        PharmacoDbClient::read_experiment_body(response(vec![b'x'; PHARMACODB_MAX_BODY_BYTES + 1]))
            .await
            .expect_err("a body over 32 MiB is refused");

    let message = format!("{error:?}");
    assert_eq!(error.code(), "api");
    assert!(
        message.contains(&format!("{PHARMACODB_MAX_BODY_BYTES}")),
        "got: {message}"
    );
    assert!(matches!(error, BioMcpError::BodyLimit { .. }), "{error}");
}

#[tokio::test]
async fn a_body_at_the_cap_still_reads() {
    let bytes =
        PharmacoDbClient::read_experiment_body(response(vec![b'y'; PHARMACODB_MAX_BODY_BYTES]))
            .await
            .expect("a body at the cap reads");

    assert_eq!(bytes.len(), PHARMACODB_MAX_BODY_BYTES);
}

#[test]
fn the_cap_sits_above_every_measured_pharmacodb_body() {
    // K-562 at 19.4 MB and doxorubicin at 18.6 MB are the measured worst cases.
    const { assert!(PHARMACODB_MAX_BODY_BYTES > 19_400_000) };
    assert_eq!(
        PHARMACODB_BODY_LIMIT_MESSAGE,
        "PharmacoDB response exceeds 32 MiB"
    );
}
