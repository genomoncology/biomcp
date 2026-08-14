use serde::{Deserialize, Serialize};

use super::AdverseEvent;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct AdverseEventSections {
    pub include_reactions: bool,
    pub include_outcomes: bool,
    pub include_concomitant: bool,
    pub include_guidance: bool,
}

#[derive(Debug, Serialize)]
pub struct FaersSubsetReport<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    data: FaersSubsetData<'a>,
}

#[derive(Debug, Serialize)]
struct FaersSubsetData<'a> {
    report_id: &'a str,
    drug: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    reactions: Option<&'a [String]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    outcomes: Option<&'a [String]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    concomitant_medications: Option<&'a [String]>,
}

impl<'a> FaersSubsetReport<'a> {
    pub fn new(event: &'a AdverseEvent, sections: AdverseEventSections) -> Self {
        Self {
            kind: "faers",
            data: FaersSubsetData {
                report_id: &event.report_id,
                drug: &event.drug,
                reactions: sections
                    .include_reactions
                    .then_some(event.reactions.as_slice()),
                outcomes: sections
                    .include_outcomes
                    .then_some(event.outcomes.as_slice()),
                concomitant_medications: sections
                    .include_concomitant
                    .then_some(event.concomitant_medications.as_slice()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::adverse_event::parse_sections;

    #[test]
    fn typed_subset_is_bounded_and_keeps_selected_empty_arrays() {
        let event = AdverseEvent {
            report_id: "1001".into(),
            drug: "drug name".into(),
            reactions: vec!["Rash".into()],
            outcomes: Vec::new(),
            patient: Some("adult".into()),
            concomitant_medications: vec!["other drug".into()],
            reporter_type: Some("Physician".into()),
            reporter_country: Some("US".into()),
            indication: Some("lung cancer".into()),
            serious: true,
            date: Some("2025-01-01".into()),
        };
        let sections = parse_sections(&["outcomes".into(), "reactions".into(), "outcomes".into()])
            .expect("valid subset");
        let value = serde_json::to_value(FaersSubsetReport::new(&event, sections))
            .expect("typed projection serializes");

        assert_eq!(value["type"], "faers");
        assert_eq!(value["data"]["report_id"], "1001");
        assert_eq!(value["data"]["drug"], "drug name");
        assert_eq!(value["data"]["reactions"], serde_json::json!(["Rash"]));
        assert_eq!(value["data"]["outcomes"], serde_json::json!([]));
        for omitted in [
            "concomitant_medications",
            "patient",
            "reporter_type",
            "reporter_country",
            "indication",
            "serious",
            "date",
        ] {
            assert!(
                value["data"].get(omitted).is_none(),
                "unexpected {omitted}: {value}"
            );
        }
    }
}
