use super::super::*;
use serde_json::Value;

#[test]
fn recorded_ctgov_summary_reaches_the_model_and_json_in_full() {
    let response: Value = serde_json::from_str(include_str!(
        "../../../../testdata/sources/ctgov/search_phelan_limit5_20260811.json"
    ))
    .expect("recorded CTGov response");
    let record = response["studies"]
        .as_array()
        .expect("recorded studies")
        .iter()
        .find(|study| study["protocolSection"]["identificationModule"]["nctId"] == "NCT07119606")
        .expect("long recorded CTGov trial");
    let expected = record["protocolSection"]["descriptionModule"]["briefSummary"]
        .as_str()
        .expect("recorded brief summary")
        .trim();
    assert!(
        expected.len() > 500,
        "fixture must cross the display byte cap"
    );

    let study: CtGovStudy = serde_json::from_value(record.clone()).expect("CTGov study");
    let trial = from_ctgov_study(&study).expect("valid trial fixture");
    assert_eq!(trial.summary.as_deref(), Some(expected));
    assert_eq!(
        serde_json::to_value(&trial).expect("trial JSON")["summary"],
        expected
    );
}

#[test]
fn summary_normalization_keeps_trimmed_text_and_rejects_blanks() {
    assert_eq!(
        normalize_summary(Some("  Full provider summary.  ")).as_deref(),
        Some("Full provider summary.")
    );
    assert_eq!(normalize_summary(Some(" \n\t ")), None);
    assert_eq!(normalize_summary(None), None);
}
