//! Bind authored semantic conversion entries to actual product events.
//! The corpus uses contextual action labels. These explicit bindings preserve
//! each contributing occurrence and require both actions for a combined effect.
use super::MyChemHit;
use super::conversion::DrugConversion;
use serde_json::{Value, json};

pub(crate) fn assert_conversion(hits: &[&MyChemHit], wanted: &Value, boundary: &str, id: &Value) {
    let events = hits
        .iter()
        .flat_map(|hit| crate::utils::sync::recover_poison(hit.conversion.lock()).clone())
        .collect::<Vec<_>>();
    for expected in wanted.as_array().unwrap() {
        if let Some(children) = expected.get("claim_occurrences") {
            assert_conversion(hits, children, boundary, id);
            continue;
        }
        let Some(action) = expected["action"].as_str() else {
            panic!("{id}: missing action");
        };
        let matches = events
            .iter()
            .filter(|event| custody(event, expected))
            .collect::<Vec<_>>();
        let bindings: &[(&str, &str)] = match action {
            "select_and_ascii_lowercase"
            | "ascii_lowercase"
            | "trim_edge_dots_ascii_lowercase"
            | "normalize_to_empty" => &[("", "select_display")],
            "select_first_code" => &[("", "select_first_code")],
            "select_card_and_ddinter" | "select_card_and_ddinter_synonym" => {
                &[("card brands", "select"), ("DDInter synonyms", "select")]
            }
            "omit_card_keep_ddinter" => &[("card brands", "cap"), ("DDInter synonyms", "select")],
            "cap" => &[("DDInter synonyms", "cap")],
            "discard_get_row" => &[("get selection", "omit")],
            "discard_search_row" => &[("search projection", "discard")],
            "deduplicate" if boundary == "search" => &[("search deduplication", "omit_duplicate")],
            "discard" if boundary == "search" => &[("search projection", "discard")],
            "omit_unvisited_limit" => &[("search selection", "omit_limit")],
            "select" if boundary == "search" => &[("search selection", "select")],
            "select"
                if boundary == "alias"
                    && expected["stage"] == "alias eligibility and insertion" =>
            {
                &[("alias eligibility and insertion", "select")]
            }
            "deduplicate" if boundary == "EMA" => &[("EMA identity", "deduplicate")],
            "select" if boundary == "EMA" => &[("EMA identity", "select")],
            "select" => &[
                ("", "select_first_code"),
                ("", "select_display"),
                ("card brands", "select"),
            ],
            "deduplicate" => &[("get merge synonyms", "deduplicate")],
            "omit_display_candidate" => &[("", "omit_display")],
            _ => &[("", action)],
        };
        let compound = matches!(
            action,
            "select_card_and_ddinter"
                | "select_card_and_ddinter_synonym"
                | "omit_card_keep_ddinter"
        );
        let selected = bindings
            .iter()
            .map(|(stage, action)| {
                matches.iter().copied().find(|event| {
                    (stage.is_empty() || event.stage == *stage)
                        && event.action == *action
                        && target(event, expected)
                })
            })
            .collect::<Vec<_>>();
        assert!(
            if compound {
                selected.iter().all(Option::is_some)
            } else {
                selected.iter().any(Option::is_some)
            },
            "{id}: unproved complete conversion {expected}; actual occurrence events: {matches:?}"
        );
        for event in selected.into_iter().flatten() {
            assert!(
                !event.reason.is_empty(),
                "{id}: absent actual decision reason"
            );
            assert!(!event.reason.contains("SENTINEL"), "{id}: unsafe reason");
        }
    }
}
fn target(event: &DrugConversion, expected: &Value) -> bool {
    expected
        .get("target")
        .is_none_or(|value| event.target.as_ref().unwrap_or(&Value::Null) == value)
}
fn custody(event: &DrugConversion, expected: &Value) -> bool {
    let value = json!(event);
    [
        "response_digest",
        "ordinal",
        "claim_index",
        "origin",
        "lexical_text",
        "namespace",
        "source_term_type",
        "source_only",
        "source_pointer",
    ]
    .into_iter()
    .all(|key| {
        expected
            .get(key)
            .is_none_or(|wanted| value.get(key).unwrap_or(&Value::Null) == wanted)
    })
}
