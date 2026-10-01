use super::super::test_support::*;
use super::*;

#[test]
fn disease_fallback_request_records_mesh_skip_before_discover() {
    let filters = DiseaseSearchFilters {
        query: Some(" Arnold Chiari syndrome ".into()),
        source: Some(" mesh ".into()),
        ..Default::default()
    };

    let request = DiseaseFallbackRequest::new(&filters, 2, 1).expect("request");

    assert_eq!(request.query, "Arnold Chiari syndrome");
    assert_eq!(request.limit, 2);
    assert_eq!(request.offset, 1);
    assert_eq!(request.skip_reason.as_deref(), Some("source=mesh"));
    assert_eq!(
        request.discover_mode,
        crate::entities::discover::DiscoverMode::AliasFallback
    );
    assert!(!request.prefer_doid);
}

#[test]
fn disease_fallback_request_records_alias_queries_and_doid_preference() {
    let filters = DiseaseSearchFilters {
        query: Some(" chronic myeloid leukemia ".into()),
        source: Some(" doid ".into()),
        ..Default::default()
    };

    let request = DiseaseFallbackRequest::new(&filters, 1, 0).expect("request");

    assert_eq!(request.query, "chronic myeloid leukemia");
    assert!(request.skip_reason.is_none());
    assert_eq!(
        request.discover_mode,
        crate::entities::discover::DiscoverMode::AliasFallback
    );
    assert!(
        request
            .resolver_queries
            .iter()
            .any(|value| value == "chronic myeloid leukemia")
    );
    assert!(request.prefer_doid);
}

#[tokio::test]
async fn ticket_400_request_command_disease_fallback_fields_drive_discover_and_crosswalk_boundaries()
 {
    let filters = DiseaseSearchFilters {
        query: Some(" Arnold Chiari syndrome ".into()),
        source: Some(" doid ".into()),
        ..Default::default()
    };
    let request = DiseaseFallbackRequest::new(&filters, 1, 1).expect("request");
    let ols_client = crate::sources::ols4::OlsClient::new_for_test("http://127.0.0.1/ols4".into())
        .expect("ols client");
    let discover_plan = ols_client.search_request_plan(&request.resolver_queries[0]);
    let mydisease_client =
        crate::sources::mydisease::MyDiseaseClient::new_for_test("http://127.0.0.1/v1".into())
            .expect("mydisease client");
    let crosswalk_plan = mydisease_client
        .lookup_disease_by_xref_request_plan("MESH", "D001139", 5)
        .expect("crosswalk plan");

    assert_eq!(
        request.discover_mode,
        crate::entities::discover::DiscoverMode::AliasFallback
    );
    assert!(request.prefer_doid);
    assert_eq!(discover_plan.path, Some("/api/search"));
    assert!(
        discover_plan
            .query_params
            .contains(&("q", "Arnold Chiari syndrome".to_string()))
    );
    assert_eq!(crosswalk_plan.path, "/query");
    assert!(crosswalk_plan.query_params[0].1.contains("D001139"));

    let candidates = vec![
        RankedDiseaseFallbackCandidate {
            label: "first Arnold Chiari syndrome".into(),
            synonyms: vec!["Arnold Chiari syndrome".into()],
            match_tier: crate::entities::discover::MatchTier::Exact,
            confidence: crate::entities::discover::DiscoverConfidence::CanonicalId,
            source_ids: vec![DiseaseFallbackId::Crosswalk(
                DiseaseXrefKind::Mesh,
                "D001139".into(),
            )],
            original_index: 0,
        },
        RankedDiseaseFallbackCandidate {
            label: "second Arnold Chiari syndrome".into(),
            synonyms: vec!["Arnold Chiari syndrome".into()],
            match_tier: crate::entities::discover::MatchTier::Exact,
            confidence: crate::entities::discover::DiscoverConfidence::CanonicalId,
            source_ids: vec![DiseaseFallbackId::Crosswalk(
                DiseaseXrefKind::Omim,
                "207950".into(),
            )],
            original_index: 1,
        },
    ];
    let page = collect_fallback_search_page(
        &request.query,
        request.limit,
        request.offset,
        candidates,
        |source_id| async move {
            let name = match source_id {
                DiseaseFallbackId::Crosswalk(DiseaseXrefKind::Mesh, _) => {
                    "first Arnold Chiari syndrome"
                }
                DiseaseFallbackId::Crosswalk(DiseaseXrefKind::Omim, _) => {
                    "second Arnold Chiari syndrome"
                }
                DiseaseFallbackId::Crosswalk(DiseaseXrefKind::Icd10Cm, _)
                | DiseaseFallbackId::CanonicalOntology(_) => "other",
            };
            Ok(Some(DiseaseSearchResult {
                id: name.replace(' ', "-"),
                name: name.into(),
                synonyms_preview: None,
                resolved_via: None,
                source_id: None,
            }))
        },
    )
    .await
    .expect("fallback collection")
    .expect("page");

    assert_eq!(page.results.len(), 1);
    assert_eq!(page.results[0].name, "second Arnold Chiari syndrome");
}

#[test]
fn fallback_candidates_rank_specific_crosswalkable_disease_ahead_of_generic_rows() {
    let candidates = rank_disease_fallback_candidates(
        "Arnold Chiari syndrome",
        &[
            test_discover_disease_concept(
                "syndromic disease",
                Some("UMLS:C0039082"),
                &[],
                &[("OMIM", "607208")],
                crate::entities::discover::MatchTier::Exact,
                crate::entities::discover::DiscoverConfidence::CanonicalId,
            ),
            test_discover_disease_concept(
                "Arnold-Chiari malformation",
                Some("MESH:D001139"),
                &["Arnold Chiari syndrome"],
                &[("MESH", "D001139"), ("OMIM", "207950")],
                crate::entities::discover::MatchTier::Contains,
                crate::entities::discover::DiscoverConfidence::CanonicalId,
            ),
        ],
    );

    assert_eq!(candidates[0].label, "Arnold-Chiari malformation");
    assert_eq!(
        candidates[0].source_ids[0],
        DiseaseFallbackId::Crosswalk(DiseaseXrefKind::Mesh, "D001139".into())
    );
}

#[tokio::test]
async fn arnold_synonym_rescue_resolves_mesh_crosswalk_through_fixture_plan() {
    let ols_client = crate::sources::ols4::OlsClient::new_for_test("http://127.0.0.1/ols4".into())
        .expect("ols client");
    let ols_plan: crate::sources::ols4::OlsSearchRequestPlan =
        ols_client.search_request_plan("Arnold Chiari syndrome");
    let xref_plan = crate::sources::mydisease::MyDiseaseClient::lookup_disease_by_xref_plan(
        "MESH", "D001139", 5,
    )
    .expect("MESH xref request plan");

    assert_eq!(ols_plan.path, Some("/api/search"));
    assert_eq!(xref_plan.path, "query");
    assert!(
        xref_plan
            .query
            .iter()
            .any(|(key, value)| key == "q" && value.contains("D001139"))
    );

    let candidates = rank_disease_fallback_candidates(
        "Arnold Chiari syndrome",
        &[test_discover_disease_concept(
            "Arnold-Chiari malformation",
            Some("MESH:D001139"),
            &["Arnold Chiari syndrome"],
            &[("MESH", "D001139")],
            crate::entities::discover::MatchTier::Contains,
            crate::entities::discover::DiscoverConfidence::CanonicalId,
        )],
    );
    let page = collect_fallback_search_page(
        "Arnold Chiari syndrome",
        1,
        0,
        candidates,
        |source_id| async move {
            assert_eq!(
                source_id,
                DiseaseFallbackId::Crosswalk(DiseaseXrefKind::Mesh, "D001139".into())
            );
            Ok(Some(DiseaseSearchResult {
                id: "MONDO:0000115".into(),
                name: "Arnold-Chiari malformation".into(),
                synonyms_preview: None,
                resolved_via: Some("MESH crosswalk".into()),
                source_id: Some("MESH:D001139".into()),
            }))
        },
    )
    .await
    .expect("fallback page should build")
    .expect("synonym-rescue row should resolve through MESH crosswalk fixture");

    assert_eq!(page.results[0].id, "MONDO:0000115");
    assert_eq!(page.results[0].name, "Arnold-Chiari malformation");
    assert_eq!(page.results[0].source_id.as_deref(), Some("MESH:D001139"));
}

#[test]
fn fallback_candidate_source_ids_prefer_primary_then_ranked_xrefs() {
    let candidate = rank_disease_fallback_candidates(
        "Arnold Chiari syndrome",
        &[test_discover_disease_concept(
            "Arnold-Chiari malformation",
            Some("OMIM:207950"),
            &[],
            &[
                ("ICD10CM", "Q07.0"),
                ("MESH", "D001139"),
                ("OMIM", "207950"),
            ],
            crate::entities::discover::MatchTier::Exact,
            crate::entities::discover::DiscoverConfidence::CanonicalId,
        )],
    );

    assert_eq!(
        candidate[0].source_ids,
        vec![
            DiseaseFallbackId::Crosswalk(DiseaseXrefKind::Mesh, "D001139".into()),
            DiseaseFallbackId::Crosswalk(DiseaseXrefKind::Omim, "207950".into()),
            DiseaseFallbackId::Crosswalk(DiseaseXrefKind::Icd10Cm, "Q07.0".into())
        ]
    );
}

#[test]
fn fallback_rows_dedupe_by_resolved_disease_id() {
    let mut seen = HashSet::new();
    let rows = [
        DiseaseSearchResult {
            id: "MONDO:0000115".into(),
            name: "Arnold-Chiari malformation".into(),
            synonyms_preview: None,
            resolved_via: Some("MESH crosswalk".into()),
            source_id: Some("MESH:D001139".into()),
        },
        DiseaseSearchResult {
            id: "MONDO:0000115".into(),
            name: "Arnold-Chiari malformation".into(),
            synonyms_preview: None,
            resolved_via: Some("OMIM crosswalk".into()),
            source_id: Some("OMIM:207950".into()),
        },
    ];

    let deduped = rows
        .into_iter()
        .filter(|row| seen.insert(row.id.clone()))
        .collect::<Vec<_>>();

    assert_eq!(deduped.len(), 1);
    assert_eq!(deduped[0].source_id.as_deref(), Some("MESH:D001139"));
}

#[test]
fn contains_all_query_tokens_ignores_generic_suffix_terms() {
    let query_tokens = normalize_disease_text("Arnold Chiari syndrome")
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();

    assert!(contains_all_query_tokens(
        &query_tokens,
        &["Arnold-Chiari malformation".to_string()]
    ));
}

#[tokio::test]
async fn fallback_search_page_swallows_discover_errors() {
    let candidates = rank_disease_fallback_candidates(
        "Arnold Chiari syndrome",
        &[test_discover_disease_concept(
            "Arnold-Chiari malformation",
            Some("MESH:D001139"),
            &["Arnold Chiari syndrome"],
            &[("MESH", "D001139")],
            crate::entities::discover::MatchTier::Exact,
            crate::entities::discover::DiscoverConfidence::CanonicalId,
        )],
    );

    let page = collect_fallback_search_page(
        "Arnold Chiari syndrome",
        10,
        0,
        candidates,
        |_source_id| async {
            let error = reqwest::Client::new()
                .get("http://127.0.0.1:0")
                .send()
                .await
                .unwrap_err();
            assert!(error.is_connect());
            Err(BioMcpError::Http(error))
        },
    )
    .await
    .expect("fallback should degrade to no rows");

    assert!(page.is_none());
}

#[tokio::test]
async fn fallback_search_page_applies_offset_and_limit_after_dedupe() {
    let candidates = rank_disease_fallback_candidates(
        "Arnold Chiari syndrome",
        &[
            test_discover_disease_concept(
                "Arnold-Chiari malformation",
                Some("MESH:D001139"),
                &["Arnold Chiari syndrome"],
                &[("MESH", "D001139")],
                crate::entities::discover::MatchTier::Exact,
                crate::entities::discover::DiscoverConfidence::CanonicalId,
            ),
            test_discover_disease_concept(
                "Arnold Chiari syndrome",
                Some("OMIM:207950"),
                &[],
                &[("OMIM", "207950")],
                crate::entities::discover::MatchTier::Exact,
                crate::entities::discover::DiscoverConfidence::CanonicalId,
            ),
            test_discover_disease_concept(
                "Chiari malformation type II",
                Some("ICD10CM:Q07.0"),
                &["Arnold Chiari syndrome"],
                &[("ICD10CM", "Q07.0")],
                crate::entities::discover::MatchTier::Contains,
                crate::entities::discover::DiscoverConfidence::CanonicalId,
            ),
        ],
    );

    let page = collect_fallback_search_page(
        "Arnold Chiari syndrome",
        1,
        1,
        candidates,
        |source_id| async move {
            let row = match source_id {
                DiseaseFallbackId::Crosswalk(DiseaseXrefKind::Mesh, value)
                    if value == "D001139" =>
                {
                    DiseaseSearchResult {
                        id: "MONDO:0000115".into(),
                        name: "Arnold-Chiari malformation".into(),
                        synonyms_preview: None,
                        resolved_via: Some("MESH crosswalk".into()),
                        source_id: Some("MESH:D001139".into()),
                    }
                }
                DiseaseFallbackId::Crosswalk(DiseaseXrefKind::Omim, value) if value == "207950" => {
                    DiseaseSearchResult {
                        id: "MONDO:0000115".into(),
                        name: "Arnold-Chiari malformation".into(),
                        synonyms_preview: None,
                        resolved_via: Some("OMIM crosswalk".into()),
                        source_id: Some("OMIM:207950".into()),
                    }
                }
                DiseaseFallbackId::Crosswalk(DiseaseXrefKind::Icd10Cm, value)
                    if value == "Q07.0" =>
                {
                    DiseaseSearchResult {
                        id: "MONDO:0002115".into(),
                        name: "Chiari malformation type II".into(),
                        synonyms_preview: None,
                        resolved_via: Some("ICD10CM crosswalk".into()),
                        source_id: Some("ICD10CM:Q07.0".into()),
                    }
                }
                other => panic!("unexpected source id: {other:?}"),
            };
            Ok(Some(row))
        },
    )
    .await
    .expect("fallback page should build")
    .expect("offset page should retain remaining unique rows");

    assert_eq!(page.total, Some(2));
    assert_eq!(page.results.len(), 1);
    assert_eq!(page.results[0].id, "MONDO:0002115");
    assert_eq!(page.results[0].source_id.as_deref(), Some("ICD10CM:Q07.0"));
}

#[test]
fn canonical_fallback_row_ignores_not_found_and_maps_hits() {
    let not_found = canonical_fallback_row(
        "DOID:8552",
        false,
        Err(BioMcpError::NotFound {
            entity: "mydisease disease".into(),
            id: "DOID:8552".into(),
            suggestion: "try another disease ID".into(),
        }),
    )
    .expect("canonical not-found should degrade cleanly");
    assert!(not_found.is_none());

    let row = canonical_fallback_row(
        "DOID:1909",
        true,
        Ok(serde_json::from_value(serde_json::json!({
            "_id": "MONDO:0005105",
            "mondo": {
                "name": "melanoma",
                "synonym": []
            },
            "disease_ontology": {
                "name": "melanoma",
                "doid": "1909",
                "synonyms": ["melanoma"]
            }
        }))
        .expect("valid disease hit")),
    )
    .expect("canonical hit should map")
    .expect("canonical hit should produce a row");

    assert_eq!(row.id, "DOID:1909");
    assert_eq!(row.name, "melanoma");
    assert_eq!(row.resolved_via.as_deref(), Some("DOID canonical"));
    assert_eq!(row.source_id.as_deref(), Some("DOID:1909"));
}
