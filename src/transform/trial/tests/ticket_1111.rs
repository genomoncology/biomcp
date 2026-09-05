use super::super::*;
use super::ticket_1107::IntoTrialSearchTestResult;

#[test]
fn recorded_ctgov_condition_vector_matches_in_detail_and_search() {
    let study: CtGovStudy = serde_json::from_str(include_str!(
        "../../../../testdata/sources/ctgov/get_nct02576665_20260811.json"
    ))
    .expect("recorded CTGov study");
    let expected = study
        .protocol_section
        .as_ref()
        .and_then(|section| section.conditions_module.as_ref())
        .expect("recorded conditions module")
        .conditions
        .clone();

    assert_eq!(expected.len(), 12);
    assert_eq!(
        from_ctgov_study(&study)
            .expect("valid trial fixture")
            .conditions,
        expected
    );
    assert_eq!(from_ctgov_hit(&study).conditions, expected);
}

#[test]
fn recorded_nci_condition_vector_matches_in_search() {
    let response: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../testdata/sources/nci_cts/search_melanoma_20260811.json"
    ))
    .expect("recorded NCI response");
    let record = &response["data"][0];
    let expected = record["diseases"]
        .as_array()
        .expect("recorded diseases")
        .iter()
        .map(|disease| {
            disease["name"]
                .as_str()
                .expect("recorded disease name")
                .to_string()
        })
        .collect::<Vec<_>>();

    assert_eq!(expected.len(), 26);
    assert_eq!(
        from_nci_hit(record)
            .into_test_result()
            .expect("valid recorded NCI hit")
            .conditions,
        expected
    );
}

#[test]
fn condition_cell_preserves_short_lists_without_a_marker() {
    assert_eq!(
        format_conditions(&[" Melanoma ".to_string(), " ".to_string()]),
        "Melanoma"
    );
}

#[test]
fn condition_cell_reserves_complete_suffix_when_item_bound_abridges() {
    let conditions = (1..=11)
        .map(|index| format!("C{index}"))
        .collect::<Vec<_>>();
    let cell = format_conditions(&conditions);

    assert!(cell.contains("C10"));
    assert!(!cell.contains("C11"));
    assert!(cell.ends_with("… [abridged; 11 conditions total]"));
    assert!(cell.len() <= 80);
    assert!(std::str::from_utf8(cell.as_bytes()).is_ok());
}

#[test]
fn condition_cell_reserves_complete_suffix_when_byte_bound_abridges_multibyte_text() {
    let cell = format_conditions(&["Å".repeat(50)]);

    assert!(cell.ends_with("… [abridged; 1 conditions total]"));
    assert!(cell.len() <= 80);
    assert!(std::str::from_utf8(cell.as_bytes()).is_ok());
}
