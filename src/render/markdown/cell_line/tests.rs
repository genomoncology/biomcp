use super::*;
use crate::entities::cell_line::{CellLineSpecies, CellLineXrefs};

fn sample() -> CellLine {
    CellLine {
        section_outcomes: crate::entities::section_outcome::SectionOutcomes::with_keys(
            &crate::entities::source_state_registry::outcome_keys("cell_line"),
        ),
        source: "Cellosaurus".to_string(),
        data_as_of: "56.0 (2026-06-25)".to_string(),
        data_as_of_kind: "release".to_string(),
        accession: "CVCL_2119".to_string(),
        secondary_accessions: Vec::new(),
        rrid: "RRID:CVCL_2119".to_string(),
        name: "MOLM-13".to_string(),
        synonyms: vec!["MOLM13".to_string()],
        species: vec![CellLineSpecies {
            taxon_id: "9606".to_string(),
            label: "Homo sapiens (Human)".to_string(),
        }],
        diseases: Vec::new(),
        category: Some("Cancer cell line".to_string()),
        sex: Some("Male".to_string()),
        age: Some("20Y".to_string()),
        resolved_from: None,
        variants: Vec::new(),
        xrefs: None,
        chembl: None,
    }
}

#[test]
fn the_card_ends_with_the_attribution_line() {
    let rendered = cell_line_markdown(&sample(), &[]).expect("renders");
    assert!(
        rendered.trim_end().ends_with(
            "Cellosaurus 56.0 (2026-06-25), CC BY 4.0. Cite Bairoch A. J. Biomol. Tech. 29:25-38 (2018)."
        ),
        "{rendered}"
    );
}

#[test]
fn the_card_offers_the_two_section_commands() {
    let rendered = cell_line_markdown(&sample(), &[]).expect("renders");
    assert!(
        rendered.contains("biomcp get cell-line CVCL_2119 variants"),
        "{rendered}"
    );
    assert!(
        rendered.contains("biomcp get cell-line CVCL_2119 xrefs"),
        "{rendered}"
    );
}

#[test]
fn the_card_names_the_accession_the_rrid_and_the_species() {
    let rendered = cell_line_markdown(&sample(), &[]).expect("renders");
    assert!(rendered.contains("RRID:CVCL_2119"), "{rendered}");
    assert!(rendered.contains("NCBI:txid9606"), "{rendered}");
}

#[test]
fn the_xrefs_section_prints_every_key_including_the_empty_ones() {
    let mut cell_line = sample();
    cell_line.xrefs = Some(CellLineXrefs {
        depmap: vec!["ACH-000362".to_string()],
        ..Default::default()
    });
    let rendered = cell_line_markdown(&cell_line, &["xrefs".to_string()]).expect("renders");
    assert!(rendered.contains("| DepMap | ACH-000362 |"), "{rendered}");
    assert!(rendered.contains("| GDSC | - |"), "{rendered}");
}

#[test]
fn search_markdown_shows_the_match_columns_and_the_attribution() {
    let rows = vec![crate::entities::cell_line::CellLineSearchResult {
        accession: "CVCL_0064".to_string(),
        name: "MV4-11".to_string(),
        species: vec![CellLineSpecies {
            taxon_id: "9606".to_string(),
            label: "Homo sapiens (Human)".to_string(),
        }],
        category: Some("Cancer cell line".to_string()),
        disease: Some("Adult acute monocytic leukemia".to_string()),
        match_kind: "exact".to_string(),
        matched_on: Some("name".to_string()),
    }];
    let rendered = cell_line_search_markdown_with_footer(
        "MV4;11",
        &rows,
        Some(1),
        &[],
        "56.0 (2026-06-25)",
        "release",
        "",
    )
    .expect("renders");
    assert!(rendered.contains("CVCL_0064"), "{rendered}");
    assert!(rendered.contains("MV4-11"), "{rendered}");
    assert!(rendered.contains("exact"), "{rendered}");
    assert!(rendered.contains("CC BY 4.0"), "{rendered}");
}

#[test]
fn a_full_window_note_is_printed_once() {
    let rendered = cell_line_search_markdown_with_footer(
        "KO",
        &[],
        None,
        &[crate::entities::cell_line::CELL_LINE_FULL_WINDOW_NOTE.to_string()],
        "56.0 (2026-06-25)",
        "release",
        "",
    )
    .expect("renders");
    assert_eq!(
        rendered.matches("Cellosaurus returned 1000 rows").count(),
        1
    );
}

// Ticket 1214 acceptance 7: the ChEMBL section names its release.

fn with_chembl_section(records: Vec<crate::entities::cell_line::CellLineChemblRecord>) -> CellLine {
    let mut cell_line = sample();
    cell_line.section_outcomes.complete(
        "chembl",
        if records.is_empty() {
            crate::entities::section_outcome::SectionOutcome::empty("ChEMBL")
        } else {
            crate::entities::section_outcome::SectionOutcome::data("ChEMBL")
        },
    );
    cell_line.chembl = Some(crate::entities::cell_line::chembl::CellLineChembl {
        source: "ChEMBL".to_string(),
        data_as_of: "ChEMBL_37 (2026-05-01)".to_string(),
        data_as_of_kind: "release".to_string(),
        records,
    });
    cell_line
}

fn molm13_chembl_record() -> crate::entities::cell_line::CellLineChemblRecord {
    crate::entities::cell_line::CellLineChemblRecord {
        chembl_id: "CHEMBL3706573".to_string(),
        name: "MOLM-13".to_string(),
        efo_id: None,
        clo_id: None,
        assay_count: 1408,
    }
}

#[test]
fn the_chembl_section_prints_one_row_and_the_release_attribution() {
    let cell_line = with_chembl_section(vec![molm13_chembl_record()]);
    let rendered = cell_line_markdown(&cell_line, &["chembl".to_string()]).expect("renders");

    assert!(rendered.contains("## ChEMBL"), "{rendered}");
    assert!(rendered.contains("CHEMBL3706573"), "{rendered}");
    assert!(rendered.contains("1408"), "{rendered}");
    assert!(
        rendered.contains("ChEMBL_37 (2026-05-01), CC BY-SA 3.0. ChEMBL is produced by EMBL-EBI."),
        "{rendered}"
    );
}

#[test]
fn a_cell_line_chembl_does_not_list_says_so_and_prints_no_row() {
    let cell_line = with_chembl_section(Vec::new());
    let rendered = cell_line_markdown(&cell_line, &["chembl".to_string()]).expect("renders");

    assert!(
        rendered.contains("no ChEMBL cell line lists this accession"),
        "{rendered}"
    );
    assert!(!rendered.contains("CHEMBL"), "{rendered}");
}

#[test]
fn an_unavailable_chembl_section_renders_no_row() {
    let mut cell_line = sample();
    cell_line.section_outcomes.complete(
        "chembl",
        crate::entities::section_outcome::SectionOutcome::unavailable(
            "ChEMBL did not answer the cell line lookup",
        ),
    );
    let rendered = cell_line_markdown(&cell_line, &["chembl".to_string()]).expect("renders");

    assert!(rendered.contains("unavailable"), "{rendered}");
    assert!(!rendered.contains("CHEMBL3706573"), "{rendered}");
}
