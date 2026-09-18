//! Ticket 1214: the ChEMBL cell line section, replayed from recorded fixtures.

use super::*;
use crate::entities::cell_line::{
    CELL_LINE_SECTION_NAMES, CellLine, build_cell_line, cell_line_sections, parse_sections,
};
use crate::sources::chembl::ChemblClient;
use reqwest::StatusCode;

fn chembl_records(file: &str) -> Vec<crate::sources::chembl::ChemblCellLine> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/sources/chembl")
        .join(file);
    let bytes = std::fs::read(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    ChemblClient::cell_lines_from_bytes(StatusCode::OK, &bytes).expect("fixture decodes")
}

/// A bare card the section attaches to.
fn sample_cell_line() -> CellLine {
    build_cell_line(
        &crate::sources::cellosaurus::CellosaurusRecord::default(),
        "CVCL_2119",
        None,
        cell_line_sections(false, false),
        "56.0 (2026-06-25)".to_string(),
        "release".to_string(),
    )
}

fn release() -> (String, String) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/sources/chembl/status_20260918.json");
    let bytes = std::fs::read(path).expect("status fixture");
    let release = ChemblClient::release_from_bytes(StatusCode::OK, &bytes).expect("release");
    data_as_of_from_release(Ok(release))
}

// Acceptance 2: the MOLM-13 record and its recorded assay count.

#[test]
fn the_molm13_record_carries_its_ids_and_assay_count() {
    let records = chembl_records("cell_line_cvcl_2119_20260918.json");
    let (data_as_of, kind) = release();
    let section = build_chembl_section(records, &[1408], data_as_of, kind);

    assert_eq!(section.source, "ChEMBL");
    assert_eq!(section.records.len(), 1);
    assert_eq!(section.records[0].chembl_id, "CHEMBL3706573");
    assert_eq!(section.records[0].name, "MOLM-13");
    assert_eq!(section.records[0].efo_id, None);
    assert_eq!(section.records[0].clo_id, None);
    assert_eq!(section.records[0].assay_count, 1408);
}

// Acceptance 3: K-562 answers through the accession despite the name difference.

#[test]
fn the_k562_record_answers_through_the_accession_not_the_name() {
    let records = chembl_records("cell_line_cvcl_0004_20260918.json");
    let (data_as_of, kind) = release();
    let section = build_chembl_section(records, &[8042], data_as_of, kind);

    assert_eq!(section.records.len(), 1);
    assert_eq!(section.records[0].chembl_id, "CHEMBL3308378");
    // Cellosaurus names this line K-562 and ChEMBL names it K562.
    assert_eq!(section.records[0].name, "K562");
    assert_eq!(section.records[0].efo_id.as_deref(), Some("EFO_0002067"));
    assert_eq!(section.records[0].clo_id.as_deref(), Some("CLO_0007059"));
}

// Acceptance 4: no record is `empty` and asks for no assay count.

#[test]
fn an_accession_chembl_does_not_list_makes_no_assay_request() {
    let records = chembl_records("cell_line_none_20260918.json");

    assert!(assay_count_requests(&records).is_empty());

    let (data_as_of, kind) = release();
    let section = build_chembl_section(records, &[], data_as_of, kind);
    assert!(section.records.is_empty());

    let mut cell_line = sample_cell_line();
    attach_chembl_section(&mut cell_line, Ok(section));
    let outcome = cell_line
        .section_outcomes
        .get("chembl")
        .expect("chembl key");
    assert_eq!(outcome.outcome().as_str(), "empty");
}

#[test]
fn one_record_asks_for_exactly_one_assay_count() {
    let records = chembl_records("cell_line_cvcl_2119_20260918.json");

    assert_eq!(assay_count_requests(&records), vec!["CHEMBL3706573"]);
}

// Acceptance 5: `all` does not include the ChEMBL section.

#[test]
fn the_all_selector_leaves_the_chembl_section_out() {
    let all = parse_sections(&["all".to_string()]).unwrap();
    assert!(!all.include_chembl);
    assert!(all.include_variants);
    assert!(all.include_xrefs);

    let asked = parse_sections(&["chembl".to_string()]).unwrap();
    assert!(asked.include_chembl);
    assert!(!asked.include_variants);
}

// Acceptance 6: the section list names it.

#[test]
fn the_section_list_names_chembl() {
    assert!(CELL_LINE_SECTION_NAMES.contains(&"chembl"));
    assert!(crate::cli::list::catalog::sections("cell-line").contains(&"chembl"));
}

// Acceptance 7: the release drives data_as_of, and a failed status degrades.

#[test]
fn the_release_fixture_sets_data_as_of_and_its_kind() {
    let (data_as_of, kind) = release();

    assert_eq!(data_as_of, "ChEMBL_37 (2026-05-01)");
    assert_eq!(kind, "release");
    assert_eq!(
        chembl_attribution(&data_as_of, &kind),
        "ChEMBL_37 (2026-05-01), CC BY-SA 3.0. ChEMBL is produced by EMBL-EBI."
    );
}

#[test]
fn a_failed_status_call_falls_back_to_the_retrieval_time() {
    let (data_as_of, kind) = data_as_of_from_release(Err(crate::error::BioMcpError::Api {
        api: "chembl".to_string(),
        message: "status is down".to_string(),
    }));

    assert_eq!(kind, "retrieved");
    assert!(data_as_of.contains('T'), "got: {data_as_of}");

    // The records still render.
    let records = chembl_records("cell_line_cvcl_2119_20260918.json");
    let section = build_chembl_section(records, &[1408], data_as_of.clone(), kind.clone());
    assert_eq!(section.records.len(), 1);
    assert_eq!(section.data_as_of_kind, "retrieved");
    assert!(
        chembl_attribution(&data_as_of, &kind).starts_with("ChEMBL, retrieved "),
        "got: {}",
        chembl_attribution(&data_as_of, &kind)
    );
}

// A transport or decode failure completes the section as `unavailable`.

#[test]
fn a_failed_lookup_completes_the_section_as_unavailable() {
    let mut cell_line = sample_cell_line();
    attach_chembl_section(
        &mut cell_line,
        Err(crate::error::BioMcpError::Api {
            api: "chembl".to_string(),
            message: "cell_line.json is down".to_string(),
        }),
    );

    let outcome = cell_line
        .section_outcomes
        .get("chembl")
        .expect("chembl key");
    assert_eq!(outcome.outcome().as_str(), "unavailable");
    assert!(cell_line.chembl.is_none());
}

#[test]
fn a_record_completes_the_section_as_data() {
    let records = chembl_records("cell_line_cvcl_2119_20260918.json");
    let (data_as_of, kind) = release();
    let section = build_chembl_section(records, &[1408], data_as_of, kind);
    let mut cell_line = sample_cell_line();
    attach_chembl_section(&mut cell_line, Ok(section));

    let outcome = cell_line
        .section_outcomes
        .get("chembl")
        .expect("chembl key");
    assert_eq!(outcome.outcome().as_str(), "data");
    assert_eq!(outcome.sources(), ["ChEMBL"]);
    assert_eq!(
        cell_line.chembl.as_ref().expect("section").records[0].assay_count,
        1408
    );
}
