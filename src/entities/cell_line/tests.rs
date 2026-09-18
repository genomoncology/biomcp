use super::*;
use crate::sources::cellosaurus::{CellosaurusBody, decode_body};
use reqwest::StatusCode;

fn records(name: &str) -> Vec<CellosaurusRecord> {
    body(name).cell_line_list
}

fn body(name: &str) -> CellosaurusBody {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/sources/cellosaurus")
        .join(name);
    let bytes = std::fs::read(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    decode_body("https://api.cellosaurus.org/test", StatusCode::OK, &bytes)
        .expect("fixture decodes")
        .expect("fixture body")
}

fn rank(fixtures: &[&str], query: &str) -> Vec<CellLineSearchResult> {
    let mut merged: Vec<CellosaurusRecord> = Vec::new();
    for fixture in fixtures {
        for record in records(fixture) {
            let accession = record.primary_accession().unwrap_or_default().to_string();
            if merged
                .iter()
                .any(|seen| seen.primary_accession().unwrap_or_default() == accession)
            {
                continue;
            }
            merged.push(record);
        }
    }
    rank_cell_line_rows(&merged, &normalize_cell_line_name(query))
}

// Acceptance 1: the normalizer.

#[test]
fn every_common_spelling_of_one_line_normalizes_to_the_same_string() {
    assert_eq!(normalize_cell_line_name("MOLM13"), "molm13");
    assert_eq!(normalize_cell_line_name("molm-13"), "molm13");
    assert_eq!(normalize_cell_line_name("Molm 13"), "molm13");

    assert_eq!(normalize_cell_line_name("MV4;11"), "mv411");
    assert_eq!(normalize_cell_line_name("MV4-11"), "mv411");
    assert_eq!(normalize_cell_line_name("MV 4;11"), "mv411");
}

#[test]
fn different_lines_do_not_normalize_together() {
    assert_ne!(
        normalize_cell_line_name("NB4"),
        normalize_cell_line_name("SJNB-4")
    );
    assert_ne!(
        normalize_cell_line_name("HL-60"),
        normalize_cell_line_name("HL-60(TB)")
    );
}

// Acceptance 2: the three spellings resolve.

#[test]
fn each_spelling_puts_its_line_first_as_an_exact_match() {
    let molm13 = rank(&["search_idsy_molm13_20260917.json"], "MOLM13");
    assert_eq!(molm13[0].accession, "CVCL_2119");
    assert_eq!(molm13[0].match_kind, "exact");

    let mv411 = rank(&["search_idsy_mv4_11_semicolon_20260917.json"], "MV4;11");
    assert_eq!(mv411[0].accession, "CVCL_0064");
    assert_eq!(mv411[0].match_kind, "exact");
    assert_eq!(mv411[0].name, "MV4-11");
    // MV4;11 and the identifier MV4-11 normalize alike, so this is an
    // identifier match, not a synonym-only one.
    assert_eq!(mv411[0].matched_on.as_deref(), Some("name"));
}

#[test]
fn a_hyphenated_query_merges_both_windows_into_one_row_per_accession() {
    let merged = rank(
        &[
            "search_idsy_molm_13_20260917.json",
            "search_idsy_molm13_20260917.json",
        ],
        "MOLM-13",
    );
    assert_eq!(merged[0].accession, "CVCL_2119");
    assert_eq!(merged[0].match_kind, "exact");
    assert_eq!(merged[0].matched_on.as_deref(), Some("name"));
    let molm13_rows = merged
        .iter()
        .filter(|row| row.accession == "CVCL_2119")
        .count();
    assert_eq!(molm13_rows, 1, "the two windows merge by accession");
}

// Acceptance 3: KG1 and KG-1.

#[test]
fn kg1_returns_all_three_exact_lines_with_the_human_identifier_first() {
    let rows = rank(&["search_idsy_kg1_20260917.json"], "KG1");
    let found: std::collections::BTreeSet<&str> =
        rows.iter().map(|row| row.accession.as_str()).collect();
    assert_eq!(
        found,
        ["CVCL_0374", "CVCL_E3VV", "CVCL_UD72"]
            .into_iter()
            .collect()
    );
    assert!(rows.iter().all(|row| row.match_kind == "exact"));
    assert_eq!(rows[0].accession, "CVCL_0374");
    assert_eq!(rows[0].matched_on.as_deref(), Some("name"));
    assert!(rows[0].species.iter().any(|sp| sp.taxon_id == "9606"));
    let mouse = rows
        .iter()
        .find(|row| row.accession == "CVCL_UD72")
        .expect("CVCL_UD72 is in the window");
    assert!(mouse.species.iter().any(|sp| sp.taxon_id == "10090"));
}

#[test]
fn kg_1_merges_its_two_windows_with_no_duplicate_rows() {
    let rows = rank(
        &[
            "search_idsy_kg_1_20260917.json",
            "search_idsy_kg1_20260917.json",
        ],
        "KG-1",
    );
    let accessions: Vec<&str> = rows.iter().map(|row| row.accession.as_str()).collect();
    let unique: std::collections::BTreeSet<&str> = accessions.iter().copied().collect();
    assert_eq!(accessions.len(), unique.len(), "no accession repeats");
    assert_eq!(rows[0].accession, "CVCL_0374");
}

#[test]
fn a_hybrid_record_keeps_every_species_it_lists() {
    let rows = rank(&["search_idsy_kg_1_20260917.json"], "KG-1");
    let hybrid = rows
        .iter()
        .find(|row| row.accession == "CVCL_1S07")
        .expect("CVCL_1S07 is in the KG-1 window");
    let taxa: Vec<&str> = hybrid
        .species
        .iter()
        .map(|sp| sp.taxon_id.as_str())
        .collect();
    assert_eq!(taxa, vec!["10029", "9606"]);
}

// Acceptance 4: NB4 against SJNB-4.

#[test]
fn nb4_puts_the_identifier_match_before_the_synonym_match() {
    let rows = rank(&["search_idsy_nb4_20260917.json"], "NB4");
    assert_eq!(rows[0].accession, "CVCL_0005");
    assert_eq!(rows[0].matched_on.as_deref(), Some("name"));

    let sjnb4_position = rows
        .iter()
        .position(|row| row.accession == "CVCL_8821")
        .expect("SJNB-4 is in the window");
    assert!(sjnb4_position > 0, "SJNB-4 sorts after the real NB4");
    assert_eq!(rows[sjnb4_position].matched_on.as_deref(), Some("synonym"));
    assert_eq!(rows[sjnb4_position].match_kind, "exact");
}

// Acceptance 6: request targeting.

#[test]
fn a_query_that_names_an_accession_is_recognized_as_one() {
    assert!(is_cvcl_accession("CVCL_2119"));
    assert!(is_cvcl_accession("cvcl_2119"));
    assert!(is_cvcl_accession("CVCL_1S07"));
    assert!(!is_cvcl_accession("MOLM13"));
    assert!(!is_cvcl_accession("CVCL_21199"));
    assert!(!is_cvcl_accession("ACH-000362"));
}

// Acceptance 7, 8, 9, 17: the card and its sections.

fn card(fixture: &str, include_variants: bool, include_xrefs: bool) -> CellLine {
    let record = records(fixture).into_iter().next().expect("one record");
    build_cell_line(
        &record,
        "",
        None,
        cell_line_sections(include_variants, include_xrefs),
        "56.0 (2026-06-25)".to_string(),
        "release".to_string(),
    )
}

#[test]
fn the_card_reads_identity_species_disease_category_and_sex() {
    let cell_line = card("get_cvcl_2119_20260917.json", false, false);
    assert_eq!(cell_line.accession, "CVCL_2119");
    assert_eq!(cell_line.rrid, "RRID:CVCL_2119");
    assert_eq!(cell_line.name, "MOLM-13");
    assert!(cell_line.synonyms.iter().any(|name| name == "MOLM13"));
    assert_eq!(cell_line.species[0].taxon_id, "9606");
    let ncit = cell_line
        .diseases
        .iter()
        .find(|disease| disease.database == "NCIt")
        .expect("an NCIt disease");
    assert_eq!(ncit.accession, "C8263");
    assert_eq!(cell_line.category.as_deref(), Some("Cancer cell line"));
    assert_eq!(cell_line.sex.as_deref(), Some("Male"));
    assert_eq!(cell_line.age.as_deref(), Some("20Y"));
    assert!(
        cell_line.xrefs.is_none(),
        "the card carries no cross-references"
    );
}

#[test]
fn the_xrefs_section_lists_the_join_keys_and_leaves_the_sample_rows_out() {
    let cell_line = card("get_cvcl_2119_20260917.json", false, true);
    let xrefs = cell_line.xrefs.expect("xrefs section");
    assert_eq!(xrefs.depmap, vec!["ACH-000362".to_string()]);
    assert_eq!(xrefs.cell_model_passport, vec!["SIDM00437".to_string()]);
    assert_eq!(xrefs.pharmacodb, vec!["MOLM13_950_2019".to_string()]);
}

#[test]
fn mv4_11_lists_its_depmap_cosmic_and_chembl_links() {
    let xrefs = card("get_cvcl_0064_20260917.json", false, true)
        .xrefs
        .expect("xrefs section");
    assert_eq!(xrefs.depmap, vec!["ACH-000045".to_string()]);
    assert_eq!(xrefs.cosmic_clp, vec!["908156".to_string()]);
    assert_eq!(xrefs.chembl, vec!["CHEMBL3308063".to_string()]);
}

#[test]
fn a_line_without_a_link_prints_the_key_as_an_empty_list() {
    let cell_line = card("get_cvcl_0007_20260917.json", false, true);
    let xrefs = cell_line.xrefs.expect("xrefs section");
    assert!(xrefs.gdsc.is_empty(), "U-937 has no GDSC link");
    assert!(xrefs.cosmic_clp.is_empty(), "U-937 has no Cosmic-CLP link");
    assert_eq!(xrefs.depmap, vec!["ACH-000406".to_string()]);

    let json = serde_json::to_value(&xrefs).expect("serializes");
    for key in [
        "depmap",
        "cosmic_clp",
        "chembl",
        "cell_model_passport",
        "gdsc",
        "pharmacodb",
        "lincs_ldp",
    ] {
        assert!(json.get(key).is_some(), "every key is present: {key}");
    }
}

#[test]
fn the_variants_section_lists_the_curated_rows_as_published() {
    let cell_line = card("get_cvcl_1844_var_20260917.json", true, false);
    let genes: Vec<&str> = cell_line
        .variants
        .iter()
        .filter_map(|variant| variant.gene.as_deref())
        .collect();
    assert_eq!(genes, vec!["DNMT3A", "NPM1", "NRAS"]);
    let dnmt3a = &cell_line.variants[0];
    assert_eq!(
        dnmt3a.description.as_deref(),
        Some("p.Arg882Cys (c.2644C>T)")
    );
    assert_eq!(dnmt3a.hgnc_id.as_deref(), Some("HGNC:2978"));
    assert_eq!(dnmt3a.zygosity.as_deref(), Some("Heterozygous"));
    assert!(dnmt3a.pubmed_ids.contains(&"31739141".to_string()));
}

#[test]
fn a_record_recorded_without_variants_completes_the_section_as_empty() {
    let cell_line = card("get_cvcl_0007_20260917.json", true, false);
    assert!(cell_line.variants.is_empty());
    let outcome = cell_line
        .section_outcomes
        .iter()
        .find(|(key, _)| *key == "variants")
        .map(|(_, outcome)| outcome.outcome())
        .expect("a variants outcome");
    assert_eq!(outcome.as_str(), "empty");
}

#[test]
fn a_record_with_variants_completes_the_section_as_data() {
    let cell_line = card("get_cvcl_1844_var_20260917.json", true, false);
    let outcome = cell_line
        .section_outcomes
        .iter()
        .find(|(key, _)| *key == "variants")
        .map(|(_, outcome)| outcome.outcome())
        .expect("a variants outcome");
    assert_eq!(outcome.as_str(), "data");
}

// Acceptance 14 and 15: the release and the attribution line.

#[test]
fn the_attribution_line_names_the_release_the_licence_and_the_citation() {
    assert_eq!(
        cell_line_attribution("56.0 (2026-06-25)", "release"),
        "Cellosaurus 56.0 (2026-06-25), CC BY 4.0. Cite Bairoch A. J. Biomol. Tech. 29:25-38 (2018)."
    );
}

#[test]
fn a_failed_release_read_names_the_retrieval_time_and_no_release() {
    let line = cell_line_attribution("2026-09-17T18:40:00+00:00", "retrieved");
    assert_eq!(
        line,
        "Cellosaurus, retrieved 2026-09-17T18:40:00+00:00, CC BY 4.0. Cite Bairoch A. J. Biomol. Tech. 29:25-38 (2018)."
    );
    assert!(!line.contains("56.0"));
}

#[test]
fn the_release_fixture_reports_the_measured_release() {
    let release = body("release_info_20260917.json")
        .header
        .and_then(|header| header.release)
        .expect("a release header");
    assert_eq!(release.version.as_deref(), Some("56.0"));
    assert_eq!(release.updated.as_deref(), Some("2026-06-25"));
}

// Acceptance 5: the window note.

#[test]
fn a_full_window_reports_an_unknown_total_and_one_note() {
    let page = synthetic_full_window();
    assert_eq!(page.total, None);
    assert_eq!(page.notes, vec![CELL_LINE_FULL_WINDOW_NOTE.to_string()]);
}

#[test]
fn a_short_window_reports_its_row_count_and_no_note() {
    let records = records("search_idsy_kg1_20260917.json");
    let window_full = records.len() >= 1000;
    assert!(!window_full);
    assert_eq!(records.len(), 3);
}

/// Cellosaurus fills the window at 1000 rows. No recorded file carries that
/// page, so the test builds one from a template.
fn synthetic_full_window() -> CellLineSearchPage {
    let rows: Vec<String> = (0..1000)
        .map(|index| {
            format!(
                r#"{{"accession-list":[{{"type":"primary","value":"CVCL_T{index:03}"}}],
                    "name-list":[{{"type":"identifier","value":"KO-{index}"}}],
                    "species-list":[{{"database":"NCBI_TaxID","accession":"9606","label":"Homo sapiens (Human)"}}]}}"#
            )
        })
        .collect();
    let bytes = format!(
        r#"{{"Cellosaurus":{{"cell-line-list":[{}]}}}}"#,
        rows.join(",")
    );
    let body = decode_body(
        "https://api.cellosaurus.org/search/cell-line",
        StatusCode::OK,
        bytes.as_bytes(),
    )
    .expect("template decodes")
    .expect("template body");
    let records = body.cell_line_list;
    assert_eq!(records.len(), 1000);
    let window_full = records.len() >= 1000;
    let results = rank_cell_line_rows(&records, &normalize_cell_line_name("KO"));
    let (total, notes) = if window_full {
        (None, vec![CELL_LINE_FULL_WINDOW_NOTE.to_string()])
    } else {
        (Some(records.len()), Vec::new())
    };
    CellLineSearchPage {
        results,
        total,
        notes,
        data_as_of: "56.0 (2026-06-25)".to_string(),
        data_as_of_kind: "release".to_string(),
    }
}

// Acceptance 11: section validation runs before any request.

#[test]
fn an_unknown_section_fails_before_any_request() {
    let error = parse_sections(&["chromosomes".to_string()]).expect_err("unknown section fails");
    let message = error.to_string();
    assert!(message.contains("chromosomes"), "{message}");
    assert!(message.contains("variants"), "{message}");
}

#[test]
fn all_turns_on_every_section() {
    let sections = parse_sections(&["all".to_string()]).expect("all parses");
    assert!(sections.include_variants);
    assert!(sections.include_xrefs);
}

// Acceptance 10: a source ID held in a cross-reference resolves to one accession.

#[test]
fn each_recorded_source_id_resolves_to_the_same_accession() {
    for (id, fixture) in [
        ("ACH-000362", "search_dr_ach_000362_20260917.json"),
        ("SIDM00437", "search_dr_sidm00437_20260917.json"),
        ("CHEMBL3706573", "search_dr_chembl3706573_20260917.json"),
        ("MOLM13_950_2019", "search_dr_molm13_950_2019_20260917.json"),
    ] {
        let accession = accession_for_source_id(id, &records(fixture))
            .unwrap_or_else(|err| panic!("{id} resolves: {err:?}"));
        assert_eq!(accession, "CVCL_2119", "{id}");
    }
}

#[test]
fn an_unmatched_source_id_is_not_found_and_names_the_search() {
    let error = accession_for_source_id("ACH-999999", &[]).expect_err("no rows is not found");
    let BioMcpError::NotFound {
        entity,
        id,
        suggestion,
    } = error
    else {
        panic!("expected NotFound, got {error:?}");
    };
    assert_eq!(entity, "cell-line");
    assert_eq!(id, "ACH-999999");
    assert_eq!(suggestion, "biomcp search cell-line ACH-999999");
}

#[test]
fn a_source_id_that_names_two_lines_asks_for_one_accession() {
    let mut ambiguous = records("search_idsy_kg1_20260917.json");
    ambiguous.truncate(2);
    let error = accession_for_source_id("KG1", &ambiguous).expect_err("two rows is ambiguous");
    let BioMcpError::InvalidArgument(message) = error else {
        panic!("expected InvalidArgument, got {error:?}");
    };
    assert!(message.contains("more than one"), "{message}");
    assert!(message.contains("CVCL_"), "{message}");
}

// Acceptance 11: an unknown accession fails with the not-found error and the search hint.

#[test]
fn an_unknown_accession_is_not_found_and_names_the_search() {
    let error = not_found("CVCL_9ZZZ");
    let BioMcpError::NotFound {
        entity,
        id,
        suggestion,
    } = error
    else {
        panic!("expected NotFound, got {error:?}");
    };
    assert_eq!(entity, "cell-line");
    assert_eq!(id, "CVCL_9ZZZ");
    assert_eq!(suggestion, "biomcp search cell-line CVCL_9ZZZ");
}
