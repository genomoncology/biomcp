use super::super::test_support::*;
use super::*;

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
            Err(BioMcpError::Api {
                api: "mydisease.info".into(),
                message: "lookup failed".into(),
            })
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

#[tokio::test]
async fn resolve_fallback_row_ignores_not_found_canonical_ids() {
    let _guard = lock_env().await;
    with_no_http_cache(async {
        let server = MockServer::start().await;
        let _env = set_env_var(
            "BIOMCP_MYDISEASE_BASE",
            Some(&format!("{}/v1", server.uri())),
        );

        Mock::given(method("GET"))
            .and(path("/v1/disease/DOID:8552"))
            .respond_with(ResponseTemplate::new(404))
            .expect(1)
            .mount(&server)
            .await;

        let client = MyDiseaseClient::new().expect("client");
        let row = resolve_fallback_row(
            &client,
            false,
            &DiseaseFallbackId::CanonicalOntology("DOID:8552".into()),
        )
        .await
        .expect("canonical not-found should degrade cleanly");

        assert!(row.is_none());
    })
    .await;
}
