use super::*;

#[test]
fn parse_sections_supports_new_disease_sections() {
    let flags = parse_sections(&[
        "phenotypes".to_string(),
        "diagnostics".to_string(),
        "variants".to_string(),
        "models".to_string(),
        "prevalence".to_string(),
        "survival".to_string(),
        "funding".to_string(),
        "disgenet".to_string(),
        "all".to_string(),
    ])
    .expect("sections should parse");
    assert!(flags.include_genes);
    assert!(flags.include_pathways);
    assert!(flags.include_phenotypes);
    assert!(flags.include_diagnostics);
    assert!(flags.include_variants);
    assert!(flags.include_models);
    assert!(flags.include_prevalence);
    assert!(flags.include_survival);
    assert!(flags.include_funding);
    assert!(flags.include_civic);
    assert!(flags.include_disgenet);
    assert!(!flags.include_clinical_features);
}

#[test]
fn disease_parse_sections_accepts_diagnostics() {
    let flags = parse_sections(&["diagnostics".to_string()]).expect("diagnostics should parse");
    assert!(flags.include_diagnostics);
    assert!(!flags.include_genes);
    assert!(!flags.include_funding);
    assert!(!flags.include_disgenet);
    assert!(!flags.include_clinical_features);
}

#[test]
fn parse_sections_accepts_clinical_features() {
    let flags =
        parse_sections(&["clinical_features".to_string()]).expect("clinical_features should parse");
    assert!(flags.include_clinical_features);
    assert!(!flags.include_genes);
    assert!(!flags.include_pathways);
    assert!(!flags.include_phenotypes);
    assert!(!flags.include_diagnostics);
    assert!(!flags.include_variants);
    assert!(!flags.include_models);
    assert!(!flags.include_prevalence);
    assert!(!flags.include_survival);
    assert!(!flags.include_funding);
    assert!(!flags.include_civic);
    assert!(!flags.include_disgenet);
}

#[test]
fn parse_sections_all_keeps_optional_sections_opt_in() {
    let flags = parse_sections(&["all".to_string()]).expect("sections should parse");
    assert!(flags.include_survival);
    assert!(!flags.include_diagnostics);
    assert!(!flags.include_funding);
    assert!(!flags.include_disgenet);
    assert!(!flags.include_clinical_features);
}

#[test]
fn disease_parse_sections_all_keeps_diagnostics_opt_in() {
    let flags = parse_sections(&["all".to_string()]).expect("sections should parse");
    assert!(!flags.include_diagnostics);
}

#[test]
fn parse_sections_unknown_section_lists_clinical_features() {
    let err =
        parse_sections(&["not_a_section".to_string()]).expect_err("unknown section should fail");
    assert!(err.to_string().contains("clinical_features"));
}

#[test]
fn parse_sections_unknown_value_suggests_name_flag_for_multi_word_diseases() {
    let err = parse_sections_for_name(
        "chronic",
        &[
            "myeloid".to_string(),
            "leukemia".to_string(),
            "survival".to_string(),
        ],
    )
    .expect_err("ambiguous multi-word disease should fail with guidance");
    let message = err.to_string();
    assert!(message.contains("Unknown section \"myeloid\" for disease"));
    assert!(message.contains("--name \"chronic myeloid leukemia\" survival"));
}

#[test]
fn get_disease_preserves_canonical_mondo_lookup_path() {
    let plan = crate::sources::mydisease::MyDiseaseClient::get_plan("MONDO:0005105")
        .expect("canonical get plan");

    assert_eq!(plan.method, crate::sources::HttpMethod::Get);
    assert_eq!(plan.path, "disease/MONDO:0005105");
    assert!(plan.query.contains(&(
        "fields".to_string(),
        crate::sources::mydisease::MYDISEASE_GET_FIELDS.to_string()
    )));
}

#[test]
fn get_disease_resolves_mesh_and_omim_crosswalk_ids_before_fetch() {
    let mesh = crate::sources::mydisease::MyDiseaseClient::lookup_disease_by_xref_plan(
        "mesh", "D008545", 5,
    )
    .expect("mesh xref plan");
    assert_eq!(mesh.path, "query");
    assert!(mesh.query.contains(&(
        "q".to_string(),
        "(mondo.xrefs.mesh:\"D008545\" OR disease_ontology.xrefs.mesh:\"D008545\" OR umls.mesh:\"D008545\")".to_string(),
    )));

    let omim = crate::sources::mydisease::MyDiseaseClient::lookup_disease_by_xref_plan(
        "omim", "154700", 5,
    )
    .expect("omim xref plan");
    assert!(omim.query.contains(&(
        "q".to_string(),
        "(mondo.xrefs.omim:\"154700\" OR disease_ontology.xrefs.omim:\"154700\")".to_string(),
    )));
}

#[test]
fn get_disease_returns_not_found_for_unresolved_crosswalk_without_name_fallback() {
    assert!(preferred_crosswalk_hit(Vec::new()).is_none());
}
