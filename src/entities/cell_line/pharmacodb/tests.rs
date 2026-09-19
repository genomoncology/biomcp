//! Ticket 1205: the cell line drug-response section, replayed from recorded
//! fixtures and pure inputs. No network, no server.

use super::*;
use crate::entities::cell_line::{
    CELL_LINE_SECTION_NAMES, CellLine, CellLineSpecies, parse_sections,
};
use crate::entities::pharmacodb::build_counts;
use crate::entities::section_outcome::{SectionOutcomeState, SectionOutcomes};
use crate::entities::source_state_registry::outcome_keys;
use crate::sources::pharmacodb::{PHARMACODB_BODY_LIMIT_MESSAGE, PharmacoDatasetCount};

const FIXED_CLOCK: &str = "2026-09-19T12:00:00Z";

fn card() -> CellLine {
    CellLine {
        section_outcomes: SectionOutcomes::with_keys(&outcome_keys("cell_line")),
        source: "Cellosaurus".to_string(),
        data_as_of: "56.0 (2026-06-25)".to_string(),
        data_as_of_kind: "release".to_string(),
        accession: "CVCL_2119".to_string(),
        secondary_accessions: Vec::new(),
        rrid: "RRID:CVCL_2119".to_string(),
        name: "MOLM-13".to_string(),
        synonyms: Vec::new(),
        species: vec![CellLineSpecies {
            taxon_id: "9606".to_string(),
            label: "Homo sapiens (Human)".to_string(),
        }],
        diseases: Vec::new(),
        category: None,
        sex: None,
        age: None,
        resolved_from: None,
        variants: Vec::new(),
        xrefs: None,
        chembl: None,
        drug_response: None,
    }
}

fn molm13_counts() -> PharmacoDbCounts {
    build_counts(
        1248,
        vec![
            PharmacoDatasetCount {
                name: "CTRPv2".to_string(),
                count: 416,
            },
            PharmacoDatasetCount {
                name: "gCSI".to_string(),
                count: 35,
            },
            PharmacoDatasetCount {
                name: "GDSC1".to_string(),
                count: 426,
            },
            PharmacoDatasetCount {
                name: "GDSC2".to_string(),
                count: 240,
            },
        ],
        FIXED_CLOCK.to_string(),
    )
}

fn outcome_of(card: &CellLine) -> SectionOutcomeState {
    card.section_outcomes
        .get("drug_response")
        .expect("the registered drug_response state")
        .outcome()
}

// Acceptance 2: counts attach as data and the section carries no rows.

#[test]
fn counts_attach_as_data_and_carry_no_rows() {
    let mut card = card();
    attach_drug_response_section(&mut card, Ok(CellLineDrugResponse::Counts(molm13_counts())));

    assert_eq!(outcome_of(&card), SectionOutcomeState::Data);
    let section = card.drug_response.expect("the counts payload");
    assert_eq!(section.total, 1117);
    assert_eq!(section.datasets.len(), 4);
    let json = serde_json::to_value(&section).expect("section JSON");
    assert!(json.get("rows").is_none(), "a section lists no rows");
    assert_eq!(json["data_as_of"], FIXED_CLOCK);
    assert_eq!(json["data_as_of_kind"], "retrieved");
    assert_eq!(json["pharmacodb_id"], 1248);
}

// Acceptance 4: a miss is empty, a mismatch is unavailable and keeps no payload.

#[test]
fn a_cell_line_pharmacodb_does_not_hold_completes_the_section_as_empty() {
    let mut card = card();
    attach_drug_response_section(&mut card, Ok(CellLineDrugResponse::Missing));

    assert_eq!(outcome_of(&card), SectionOutcomeState::Empty);
    assert!(card.drug_response.is_none());
    assert_eq!(
        drug_response_empty_message(),
        "no PharmacoDB cell line for this accession"
    );
}

#[test]
fn an_accession_mismatch_is_unavailable_and_renders_no_payload() {
    let mut card = card();
    attach_drug_response_section(&mut card, Ok(CellLineDrugResponse::Mismatch));

    assert_eq!(outcome_of(&card), SectionOutcomeState::Unavailable);
    assert!(card.drug_response.is_none());
    let json = serde_json::to_value(
        card.section_outcomes
            .get("drug_response")
            .expect("the state"),
    )
    .expect("outcome JSON");
    assert_eq!(json["message"], "PharmacoDB accession does not match");
}

// Acceptance 6: a body over the cap says so on the card.

#[test]
fn a_body_over_the_cap_is_unavailable_with_the_size_message() {
    let mut card = card();
    attach_drug_response_section(
        &mut card,
        Err(BioMcpError::BodyLimit {
            source_name: "pharmacodb".to_string(),
            max_bytes: 32 * 1024 * 1024,
        }),
    );

    assert_eq!(outcome_of(&card), SectionOutcomeState::Unavailable);
    let json = serde_json::to_value(
        card.section_outcomes
            .get("drug_response")
            .expect("the state"),
    )
    .expect("outcome JSON");
    assert_eq!(json["message"], PHARMACODB_BODY_LIMIT_MESSAGE);
}

// Acceptance 7 and 8: `all` leaves the section out, and an unknown section names
// it.

#[test]
fn the_section_is_asked_for_by_name_and_all_leaves_it_out() {
    let named = parse_sections(&["drug_response".to_string()]).expect("a known section");
    assert!(named.include_drug_response);

    let all = parse_sections(&["all".to_string()]).expect("the all expansion");
    assert!(!all.include_drug_response);
    assert!(!all.include_chembl);
}

#[test]
fn an_unknown_section_lists_drug_response_for_cell_lines() {
    let error = parse_sections(&["dose".to_string()]).expect_err("dose is not a section");

    assert!(error.to_string().contains("drug_response"), "{error}");
    assert!(CELL_LINE_SECTION_NAMES.contains(&"drug_response"));
}

// Acceptance 7: under `all` the card is the same whether the payload is there or
// not, so the pre-ticket output cannot change.

#[test]
fn the_all_card_is_byte_identical_with_and_without_the_payload() {
    let mut with_payload = card();
    attach_drug_response_section(
        &mut with_payload,
        Ok(CellLineDrugResponse::Counts(molm13_counts())),
    );
    let sections = vec!["all".to_string()];

    let plain = crate::render::markdown::cell_line_markdown(&card(), &sections).expect("renders");
    let loaded =
        crate::render::markdown::cell_line_markdown(&with_payload, &sections).expect("renders");

    assert_eq!(plain, loaded);
    assert!(!plain.contains("PharmacoDB)"), "{plain}");
}

// Acceptance 9: the section heading, the counts line, the next command, and the
// one fixed attribution line.

#[test]
fn the_section_prints_the_heading_the_counts_and_the_next_command() {
    let mut card = card();
    attach_drug_response_section(&mut card, Ok(CellLineDrugResponse::Counts(molm13_counts())));
    let rendered =
        crate::render::markdown::cell_line_markdown(&card, &["drug_response".to_string()])
            .expect("renders");

    assert!(
        rendered.contains("## Drug response (PharmacoDB)"),
        "{rendered}"
    );
    assert!(
        rendered.contains("1117 experiments: CTRPv2 416, gCSI 35, GDSC1 426, GDSC2 240"),
        "{rendered}"
    );
    assert!(
        rendered.contains("biomcp cell-line drug-response CVCL_2119 --dataset CTRPv2"),
        "{rendered}"
    );
    assert!(
        rendered.contains("Values as published by PharmacoDB."),
        "{rendered}"
    );
    assert!(rendered.contains("non-commercial"), "{rendered}");
    assert!(
        rendered.contains("retrieved 2026-09-19T12:00:00Z"),
        "{rendered}"
    );
    assert!(rendered.contains("gives no units"), "{rendered}");
}

#[test]
fn an_empty_section_says_so_and_prints_no_counts_line() {
    let mut card = card();
    attach_drug_response_section(
        &mut card,
        Ok(CellLineDrugResponse::Counts(build_counts(
            1248,
            Vec::new(),
            FIXED_CLOCK.to_string(),
        ))),
    );
    let rendered =
        crate::render::markdown::cell_line_markdown(&card, &["drug_response".to_string()])
            .expect("renders");

    assert_eq!(outcome_of(&card), SectionOutcomeState::Empty);
    assert!(
        rendered.contains("no PharmacoDB cell line for this accession"),
        "{rendered}"
    );
    assert!(!rendered.contains("experiments:"), "{rendered}");
}
