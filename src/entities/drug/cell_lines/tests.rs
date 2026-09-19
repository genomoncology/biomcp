//! Ticket 1205: the drug cell-lines section and its row helper arguments. Pure:
//! recorded counts and pure inputs. No network, no server.

use super::*;
use crate::entities::drug::DRUG_SECTION_NAMES;
use crate::entities::pharmacodb::build_counts;
use crate::entities::section_outcome::SectionOutcomeState;
use crate::sources::pharmacodb::PharmacoDatasetCount;

const FIXED_CLOCK: &str = "2026-09-19T12:00:00Z";

fn venetoclax_counts() -> PharmacoDbCounts {
    build_counts(
        53572,
        vec![
            PharmacoDatasetCount {
                name: "CTRPv2".to_string(),
                count: 536,
            },
            PharmacoDatasetCount {
                name: "GDSC1".to_string(),
                count: 981,
            },
            PharmacoDatasetCount {
                name: "GDSC2".to_string(),
                count: 1052,
            },
            PharmacoDatasetCount {
                name: "NCI60".to_string(),
                count: 113,
            },
            PharmacoDatasetCount {
                name: "PRISM".to_string(),
                count: 926,
            },
        ],
        FIXED_CLOCK.to_string(),
    )
}

fn card() -> Drug {
    Drug {
        section_outcomes: crate::entities::drug::default_drug_section_outcomes(),
        name: "venetoclax".to_string(),
        drugbank_id: None,
        chembl_id: None,
        unii: None,
        drug_type: None,
        mechanism: None,
        mechanisms: Vec::new(),
        approval_date: None,
        approval_date_raw: None,
        approval_date_display: None,
        approval_summary: None,
        brand_names: Vec::new(),
        route: None,
        targets: Vec::new(),
        variant_targets: Vec::new(),
        target_family: None,
        target_family_name: None,
        indications: Vec::new(),
        interactions: Vec::new(),
        interaction_text: None,
        interaction_pagination: None,
        interaction_bundle_freshness: None,
        pharm_classes: Vec::new(),
        top_adverse_events: Vec::new(),
        faers_query: None,
        label: None,
        label_set_id: None,
        shortage: None,
        approvals: None,
        fda_orphan_designations: None,
        us_safety_warnings: None,
        ema_regulatory: None,
        ema_safety: None,
        ema_shortage: None,
        who_prequalification: None,
        civic: None,
        cell_lines: None,
    }
}

fn outcome_of(drug: &Drug) -> SectionOutcomeState {
    drug.section_outcomes
        .get("cell_lines")
        .expect("the registered cell_lines state")
        .outcome()
}

// Acceptance 5: the section reports counts and lists no rows.

#[test]
fn the_section_attaches_the_recorded_venetoclax_counts_as_data() {
    let mut drug = card();
    attach_cell_lines_section(&mut drug, Ok(DrugCellLines::Counts(venetoclax_counts())));

    assert_eq!(outcome_of(&drug), SectionOutcomeState::Data);
    let section = drug.cell_lines.expect("the counts payload");
    assert_eq!(section.pharmacodb_id, 53572);
    assert_eq!(section.total, 3608);
    let json = serde_json::to_value(&section).expect("section JSON");
    assert!(json.get("rows").is_none(), "a section lists no rows");
    assert_eq!(json["data_as_of"], FIXED_CLOCK);
    assert_eq!(json["data_as_of_kind"], "retrieved");
}

#[test]
fn a_compound_pharmacodb_does_not_hold_completes_the_section_as_empty() {
    let mut drug = card();
    attach_cell_lines_section(&mut drug, Ok(DrugCellLines::Missing));

    assert_eq!(outcome_of(&drug), SectionOutcomeState::Empty);
    assert!(drug.cell_lines.is_none());
    assert_eq!(
        cell_lines_empty_message(),
        "no PharmacoDB compound with this name"
    );
}

#[test]
fn a_body_over_the_cap_is_unavailable_with_the_size_message() {
    let mut drug = card();
    attach_cell_lines_section(
        &mut drug,
        Err(BioMcpError::BodyLimit {
            source_name: "pharmacodb".to_string(),
            max_bytes: 32 * 1024 * 1024,
        }),
    );

    assert_eq!(outcome_of(&drug), SectionOutcomeState::Unavailable);
    let json = serde_json::to_value(drug.section_outcomes.get("cell_lines").expect("the state"))
        .expect("outcome JSON");
    assert_eq!(json["message"], "PharmacoDB response exceeds 32 MiB");
}

// Acceptance 5: a listing with no filter fails before any request.

#[test]
fn a_listing_with_no_filter_fails_and_names_the_counts_command() {
    let error = DrugCellLinesRequest::from_args(None, None).expect_err("one filter is required");

    assert!(matches!(error, BioMcpError::InvalidArgument(_)));
    let message = error.to_string();
    assert!(message.contains("--cell-line"), "{message}");
    assert!(message.contains("--dataset"), "{message}");
    assert!(
        message.contains("biomcp get drug <name> cell_lines"),
        "{message}"
    );
}

#[test]
fn one_filter_is_accepted_and_two_are_refused() {
    assert!(matches!(
        DrugCellLinesRequest::from_args(Some("CVCL_2119"), None),
        Ok(DrugCellLinesRequest::Pair { .. })
    ));
    assert!(matches!(
        DrugCellLinesRequest::from_args(None, Some("GDSC1")),
        Ok(DrugCellLinesRequest::Dataset { .. })
    ));
    assert!(DrugCellLinesRequest::from_args(Some("CVCL_2119"), Some("GDSC1")).is_err());
}

// Acceptance 8: an unknown section names `cell_lines` for drugs.

#[test]
fn the_section_name_is_listed_for_drugs() {
    assert!(DRUG_SECTION_NAMES.contains(&"cell_lines"));
    assert_eq!(DRUG_SECTION_CELL_LINES, "cell_lines");
}

// Acceptance 9: the card heading, the counts line, both next commands, and the
// fixed attribution line.

#[test]
fn the_section_prints_the_heading_the_counts_and_the_next_commands() {
    let mut drug = card();
    attach_cell_lines_section(&mut drug, Ok(DrugCellLines::Counts(venetoclax_counts())));
    let rendered = crate::render::markdown::drug_markdown(&drug, &["cell_lines".to_string()])
        .expect("renders");

    assert!(
        rendered.contains("## Cell lines (PharmacoDB)"),
        "{rendered}"
    );
    assert!(
        rendered
            .contains("3608 experiments: CTRPv2 536, GDSC1 981, GDSC2 1052, NCI60 113, PRISM 926"),
        "{rendered}"
    );
    assert!(
        rendered.contains("biomcp drug cell-lines venetoclax --cell-line <CVCL accession>"),
        "{rendered}"
    );
    assert!(
        rendered.contains("biomcp drug cell-lines venetoclax --dataset CTRPv2"),
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
}

// Acceptance 7: under `all` the card never renders the section.

#[test]
fn the_all_card_is_byte_identical_with_and_without_the_payload() {
    let mut with_payload = card();
    attach_cell_lines_section(
        &mut with_payload,
        Ok(DrugCellLines::Counts(venetoclax_counts())),
    );
    let sections = vec!["all".to_string()];

    let plain = crate::render::markdown::drug_markdown(&card(), &sections).expect("renders");
    let loaded = crate::render::markdown::drug_markdown(&with_payload, &sections).expect("renders");

    assert_eq!(plain, loaded);
    assert!(!plain.contains("(PharmacoDB)"), "{plain}");
}
