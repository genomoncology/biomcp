//! Tier 2 — request construction. Pure: builds `RequestPlan`s and asserts the exact
//! method / path / query that would be sent. Nothing is sent.

use crate::error::BioMcpError;
use crate::sources::HttpMethod;
use crate::sources::mydisease::{MYDISEASE_GET_FIELDS, MYDISEASE_SEARCH_FIELDS, MyDiseaseClient};

fn q(plan: &crate::sources::RequestPlan) -> &str {
    plan.query_value("q").expect("q present")
}

#[test]
fn query_plan_sets_search_shape() {
    let plan = MyDiseaseClient::query_plan(
        " melanoma ",
        10,
        2,
        Some("mesh"),
        Some("dominant"),
        Some("HP:0001250"),
        Some("adult"),
    )
    .unwrap();

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "query");
    assert_eq!(plan.query_value("size"), Some("10"));
    assert_eq!(plan.query_value("from"), Some("2"));
    assert_eq!(plan.query_value("fields"), Some(MYDISEASE_SEARCH_FIELDS));
    let query = q(&plan);
    assert!(query.contains("disease_ontology.name:melanoma"));
    assert!(query.contains("umls.mesh:*"));
    assert!(query.contains("hpo.inheritance.hpo_name:*dominant*"));
    assert!(query.contains("hpo.phenotype_related_to_disease.hpo_id:*HP\\:0001250*"));
    assert!(query.contains("hpo.clinical_course.hpo_name:*adult*"));
}

#[test]
fn query_plan_retrieves_exact_synonym_leaf_fields_for_abbreviations() {
    // MyDisease indexes `mondo.synonym` and `disease_ontology.synonyms` as
    // objects scoped by synonym type, so only the `exact` leaf fields hold
    // tokens. The parent paths match nothing and hid every exact synonym and
    // abbreviation from the candidate set (ticket 1295).
    let plan = MyDiseaseClient::query_plan("NSCLC", 15, 0, None, None, None, None).unwrap();

    assert_eq!(
        q(&plan),
        "(disease_ontology.name:NSCLC OR disease_ontology.synonyms.exact:NSCLC OR mondo.name:NSCLC OR mondo.synonym.exact:NSCLC)"
    );
}

#[test]
fn query_plan_builds_id_lookup_shape() {
    let plan = MyDiseaseClient::query_plan("MONDO:0005105", 1, 0, None, None, None, None).unwrap();

    assert_eq!(
        q(&plan),
        "(_id:\"MONDO\\:0005105\" OR disease_ontology.doid:\"MONDO\\:0005105\")"
    );
}

#[test]
fn xref_plan_builds_crosswalk_shapes() {
    let mesh_plan = MyDiseaseClient::lookup_disease_by_xref_plan("mesh", "D008545", 5).unwrap();
    let omim_plan = MyDiseaseClient::lookup_disease_by_xref_plan("omim", "154700", 5).unwrap();
    let icd10_plan = MyDiseaseClient::lookup_disease_by_xref_plan("icd10cm", "Q07.0", 5).unwrap();

    assert_eq!(icd10_plan.method, HttpMethod::Get);
    assert_eq!(icd10_plan.path, "query");
    assert_eq!(icd10_plan.query_value("size"), Some("5"));
    assert_eq!(icd10_plan.query_value("from"), Some("0"));
    assert_eq!(
        icd10_plan.query_value("fields"),
        Some(MYDISEASE_SEARCH_FIELDS)
    );
    assert_eq!(
        q(&mesh_plan),
        "(mondo.xrefs.mesh:\"D008545\" OR disease_ontology.xrefs.mesh:\"D008545\" OR umls.mesh:\"D008545\")"
    );
    assert_eq!(
        q(&omim_plan),
        "(mondo.xrefs.omim:\"154700\" OR disease_ontology.xrefs.omim:\"154700\")"
    );
    assert_eq!(
        q(&icd10_plan),
        "(mondo.xrefs.icd10:\"Q07.0\" OR mondo.xrefs.icd10:\"ICD10:Q07.0\" OR disease_ontology.xrefs.icd10:\"Q07.0\" OR disease_ontology.xrefs.icd10:\"ICD10:Q07.0\" OR umls.icd10am:\"Q07.0\" OR umls.icd10am:\"ICD10:Q07.0\")"
    );
}

#[test]
fn get_plan_sets_path_and_fields() {
    let plan = MyDiseaseClient::get_plan(" MONDO:0005105 ").unwrap();

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "disease/MONDO:0005105");
    assert_eq!(plan.query_value("fields"), Some(MYDISEASE_GET_FIELDS));
}

#[test]
fn get_plan_rejects_path_query_separators_before_network() {
    for id in [
        "MONDO:0005105/extra",
        "MONDO:0005105\\extra",
        "MONDO:0005105?fields=_id",
        "MONDO:0005105#fragment",
    ] {
        assert!(matches!(
            MyDiseaseClient::get_plan(id),
            Err(BioMcpError::InvalidArgument(_))
        ));
    }
}

#[test]
fn request_plans_preserve_validation_before_network() {
    assert!(matches!(
        MyDiseaseClient::query_plan(" ", 10, 0, None, None, None, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        MyDiseaseClient::lookup_disease_by_xref_plan("mesh", " ", 5),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        MyDiseaseClient::get_plan(" "),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        MyDiseaseClient::query_plan("melanoma", 40, 9_980, None, None, None, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        MyDiseaseClient::lookup_disease_by_xref_plan("mesh", "D008545", 10_001),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        MyDiseaseClient::get_plan(&"x".repeat(129)),
        Err(BioMcpError::InvalidArgument(_))
    ));
}

#[test]
fn legacy_plan_helpers_keep_entity_tests_stable() {
    let client = MyDiseaseClient::new_for_test("http://127.0.0.1/v1".into()).unwrap();
    let plan = client
        .query_request_plan("melanoma", 10, 0, None, None, None, None)
        .unwrap();
    assert_eq!(plan.path, "/query");
    assert!(plan.query_params.contains(&("size", "10".to_string())));

    let get = client.get_request_plan("MONDO:0005105").unwrap();
    assert_eq!(get.path, "/disease/MONDO:0005105");
    assert_eq!(
        get.query_params,
        vec![("fields", MYDISEASE_GET_FIELDS.to_string())]
    );
}
