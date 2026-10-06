//! Sidecar tests for variant MyVariant search helpers.

use super::super::{VariantProteinAlias, VariantSearchFilters, VariantSearchResult};
use super::*;
#[test]
fn interval_annotation_comparison_table() {
    super::super::resolution::tests::interval::annotation_table(
        annotation_matches_request,
        |request, hit, seen, retained| retain_compatible_hits(request, [hit], seen, retained),
        finalize_exact_page,
    );
}

#[test]
fn search_query_summary_includes_hgvsc_and_rsid() {
    let summary = search_query_summary(&VariantSearchFilters {
        gene: Some("BRAF".into()),
        hgvsc: Some("c.1799T>A".into()),
        rsid: Some("rs113488022".into()),
        ..Default::default()
    });
    assert_eq!(summary, "gene=BRAF, hgvsc=c.1799T>A, rsid=rs113488022");
}

#[test]
fn search_query_summary_includes_residue_alias_marker() {
    let summary = search_query_summary(&VariantSearchFilters {
        gene: Some("PTPN22".into()),
        protein_alias: Some(VariantProteinAlias {
            position: 620,
            residue: 'W',
        }),
        ..Default::default()
    });
    assert_eq!(summary, "gene=PTPN22, residue_alias=620W");
}

#[test]
fn filter_statuses_describe_evaluation_and_keep_canonical_order() {
    let filters = VariantSearchFilters {
        gene: Some("MISSING".into()),
        hgvsp: Some("p.V1A".into()),
        protein_alias: Some(VariantProteinAlias {
            position: 1,
            residue: 'A',
        }),
        ..Default::default()
    };
    let diagnostics = vec![SearchDiagnostic::GeneUnavailable {
        requested: "MISSING".into(),
    }];

    assert_eq!(
        serde_json::to_value(filter_evaluation(&filters, &diagnostics))
            .expect("filter statuses serialize"),
        serde_json::json!({
            "gene": "unavailable",
            "hgvsp": "evaluated",
            "residue_alias": "evaluated"
        })
    );
    assert_eq!(
        serde_json::to_value(filter_evaluation(&VariantSearchFilters::default(), &[]))
            .expect("empty filter statuses serialize"),
        serde_json::json!({})
    );
}

#[test]
fn resolution_status_follows_compatible_indeterminate_and_exhaustive_truth_table() {
    assert_eq!(
        resolution_status(1, false, true),
        VariantResolutionStatus::Resolved
    );
    assert_eq!(
        resolution_status(2, false, true),
        VariantResolutionStatus::Ambiguous
    );
    assert_eq!(
        resolution_status(0, true, true),
        VariantResolutionStatus::Ambiguous
    );
    assert_eq!(
        resolution_status(1, false, false),
        VariantResolutionStatus::Ambiguous
    );
    assert_eq!(
        resolution_status(0, false, true),
        VariantResolutionStatus::Unresolved
    );
}

#[test]
fn shared_article_and_protein_completion_policy_keeps_empty_page_or_known_total() {
    assert!(!candidate_scan_exhaustive(Some(10), 5, 5));
    assert!(!candidate_scan_exhaustive(None, 5, 5));
    assert!(candidate_scan_exhaustive(Some(5), 5, 5));
    assert!(candidate_scan_exhaustive(Some(10), 5, 0));
}

#[test]
fn normalized_source_identity_key_deduplicates_alias_order_and_spelling() {
    let first = SourceVariantIdentity {
        genomic_id: "chr7:g.1A>T".into(),
        genome_build: "GRCh37".into(),
        genes: vec!["BRAF".into()],
        protein_changes: vec!["p.Val600Glu".into(), "p.V600E".into()],
        coding_changes: vec!["NM_004333.6:c.1799T>A".into()],
        rsids: Vec::new(),
    };
    let second = SourceVariantIdentity {
        protein_changes: vec!["V600E".into()],
        coding_changes: vec!["c.1799T>A".into()],
        ..first.clone()
    };
    assert_eq!(first.normalized_key(), second.normalized_key());
}

#[test]
fn exact_aggregation_retains_identical_complex_protein_hgvs() {
    let requested = RequestedVariantIdentity::for_search(
        Some("EGFR".into()),
        Some("p.Glu746_Ala750del".into()),
        None,
        None,
    );
    let source_hit = crate::sources::myvariant::MyVariantHit::from_value(serde_json::json!({
        "_id": "chr7:g.55242465_55242479del",
        "dbnsfp": {
            "genename": "EGFR",
            "hgvsp": "NP_005219.2:p.Glu746_Ala750del"
        },
        "snpeff": {"ann": [{
            "feature_id": "NM_005228.5",
            "genename": "EGFR",
            "hgvs_c": "c.2235_2249del",
            "hgvs_p": "NP_005219.2:p.Glu746_Ala750del"
        }]}
    }))
    .expect("valid MyVariant hit");
    let mut seen = HashSet::new();
    let mut retained = Vec::new();

    assert!(!retain_compatible_hits(
        &requested,
        [source_hit],
        &mut seen,
        &mut retained,
    ));
    assert_eq!(
        retained[0].row.matched_alias.as_deref(),
        Some("NP_005219.2:p.Glu746_Ala750del")
    );
    let page = finalize_exact_page(&requested, retained, 0, 10, false, true);
    assert!(
        page.results[0].transcript_annotations.as_ref().unwrap()[0]
            .roles
            .iter()
            .any(|role| matches!(role, TranscriptAnnotationRole::Matched))
    );
}

#[test]
fn exact_projection_keeps_paired_snpeff_roles_separate_from_dbnsfp_match() {
    let corpus: serde_json::Value =
        serde_json::from_str(include_str!("../resolution/genomic_oracles/cases.json")).unwrap();
    for case in corpus["search_cases"].as_array().unwrap() {
        let requested: RequestedVariantIdentity =
            serde_json::from_value(case["requested"].clone()).unwrap();
        let observations = &case["requested_observations"];
        let identity = serde_json::to_value(&requested).unwrap();
        for (field, value) in observations["identity"].as_object().unwrap() {
            assert_eq!(identity[field], *value);
        }
        assert_eq!(
            serde_json::to_value(requested.normalized_aliases()).unwrap(),
            observations["normalized_aliases"]
        );
        assert_eq!(requested.human_label(), observations["human_label"]);
        let mut seen = HashSet::new();
        let mut retained = Vec::new();
        let hits: Vec<MyVariantHit> = case["hits"].as_array().unwrap().iter().cloned().map(MyVariantHit::from_value).collect::<Result<_, _>>().unwrap();
        assert!(!retain_compatible_hits(
            &requested,
            hits,
            &mut seen,
            &mut retained
        ));
        let keys: HashSet<String> = serde_json::from_value(case["source_keys"].clone()).unwrap();
        assert_eq!(seen, keys);
        let page = finalize_exact_page(&requested, retained, 0, 5, false, true);
        assert_eq!(
            serde_json::to_value(&page.results).unwrap(),
            case["expected_results"],
            "{}",
            case["id"]
        );
    }
    let requested = RequestedVariantIdentity::for_search(
        Some("HSD17B4".into()),
        Some("H540R".into()),
        None,
        None,
    );
    let source_hit: MyVariantHit = MyVariantHit::from_value(serde_json::json!({
        "_id": "chr5:g.118860951A>G",
        "dbnsfp": {"genename": "HSD17B4", "hgvsp": ["p.His515Arg", "p.His540Arg"]},
        "clinvar": {"rcv": {"preferred_name": "NM_000414.3(HSD17B4):c.1544A>G (p.His515Arg)"}},
        "snpeff": {"ann": [
            {"feature_id":"NM_001199291.2", "genename":"HSD17B4", "hgvs_c":"c.1619A>G", "hgvs_p":"p.His540Arg"},
            {"feature_id":"NM_000414.3", "genename":"HSD17B4", "hgvs_c":"c.1544A>G", "hgvs_p":"p.His515Arg"},
            {"feature_id":"XM_1", "genename":"HSD17B4", "hgvs_c":"c.1A>G", "hgvs_p":null},
            {"feature_id":"NM_000414.3", "genename":"HSD17B4", "hgvs_c":"c.1544A>G", "hgvs_p":"p.His515Arg"}
        ]}
    }))
    .expect("valid paired fixture");
    let mut seen = HashSet::new();
    let mut retained = Vec::new();
    assert!(!retain_compatible_hits(
        &requested,
        [source_hit],
        &mut seen,
        &mut retained,
    ));

    let page = finalize_exact_page(&requested, retained, 0, 10, false, true);
    let row = &page.results[0];
    assert_eq!(row.matched_alias.as_deref(), Some("p.His540Arg"));
    assert_eq!(row.transcript.as_deref(), Some("NM_000414.3"));
    assert_eq!(row.transcript_annotations_complete, Some(true));
    let annotations = row
        .transcript_annotations
        .as_ref()
        .expect("exact annotations");
    assert_eq!(annotations.len(), 3);
    assert_eq!(annotations[0].transcript.as_deref(), Some("NM_000414.3"));
    assert_eq!(
        serde_json::to_value(&annotations[0].roles).unwrap(),
        serde_json::json!(["displayed"])
    );
    assert_eq!(annotations[1].transcript.as_deref(), Some("NM_001199291.2"));
    assert_eq!(
        serde_json::to_value(&annotations[1].roles).unwrap(),
        serde_json::json!(["matched"])
    );
    assert_eq!(annotations[2].transcript.as_deref(), Some("XM_1"));
    assert!(annotations[2].roles.is_empty());
    assert_eq!(
        serde_json::to_value(&annotations[2]).unwrap(),
        serde_json::json!({
            "source":"myvariant.info/snpeff.ann",
            "gene":"HSD17B4",
            "transcript":"XM_1",
            "hgvs_c":"c.1A>G",
            "hgvs_p":null,
            "roles":[]
        })
    );
}

#[test]
fn split_snpeff_fields_do_not_inherit_dbnsfp_match_role() {
    let requested = RequestedVariantIdentity::for_search(
        Some("BRAF".into()),
        Some("V600E".into()),
        Some("c.1799T>A".into()),
        None,
    );
    let source_hit: MyVariantHit = MyVariantHit::from_value(serde_json::json!({
        "_id": "chr7:g.140453136A>T",
        "dbnsfp": {"genename":"BRAF", "hgvsp":"p.V600E", "hgvsc":"c.1799T>A"},
        "snpeff": {"ann": [
            {"feature_id":"NM_1", "genename":"BRAF", "hgvs_c":"c.1799T>A"},
            {"feature_id":"NM_2", "genename":"BRAF", "hgvs_p":"p.V600E"}
        ]}
    }))
    .unwrap();
    let mut seen = HashSet::new();
    let mut retained = Vec::new();
    retain_compatible_hits(&requested, [source_hit], &mut seen, &mut retained);
    let page = finalize_exact_page(&requested, retained, 0, 10, false, true);
    assert!(
        page.results[0]
            .transcript_annotations
            .as_ref()
            .unwrap()
            .iter()
            .all(|annotation| !annotation
                .roles
                .iter()
                .any(|role| matches!(role, TranscriptAnnotationRole::Matched)))
    );
}

#[test]
fn matched_role_requires_one_complete_transcript_specific_tuple() {
    let rows: serde_json::Value =
        serde_json::from_str(include_str!("../resolution/coding_oracles/aliases.json")).unwrap();
    assert_eq!(rows["annotations"].as_array().unwrap().len(), 69);
    let mut predicates = 0;
    for row in rows["annotations"].as_array().unwrap() {
        let requested: RequestedVariantIdentity =
            serde_json::from_value(row["request"].clone()).unwrap();
        if let Some(input) = match row["id"].as_str().unwrap() {
            "RETAINED03" => Some("rs123"),
            "RETAINED04" => Some("chr1:g.1A>G"),
            _ => None,
        } {
            assert_eq!(
                RequestedVariantIdentity::from_variant_input(input).unwrap(),
                requested
            );
        }
        let annotations = row
            .get("annotations")
            .cloned()
            .unwrap_or_else(|| serde_json::json!([row["annotation"]]));
        let expected = if row["expected"].is_array() {
            row["expected"].clone()
        } else {
            serde_json::json!([row["expected"]])
        };
        assert_eq!(
            annotations.as_array().unwrap().len(),
            expected.as_array().unwrap().len()
        );
        for (annotation, expected) in annotations
            .as_array()
            .unwrap()
            .iter()
            .zip(expected.as_array().unwrap())
        {
            let hit: MyVariantHit = MyVariantHit::from_value(serde_json::json!({
                "_id":"x", "snpeff":{"ann":annotation}
            }))
            .unwrap();
            let annotation = &hit.source().snpeff().unwrap().annotations()[0];
            assert_eq!(
                annotation_matches_request(&annotation, &requested),
                expected.as_bool().unwrap(),
                "{}",
                row["id"]
            );
            predicates += 1;
        }
    }
    assert_eq!(predicates, 70);
}

#[test]
fn transcript_annotation_page_budget_is_all_or_nothing_at_256_kib() {
    fn retained(id: usize, annotations: usize, field_len: usize) -> RetainedVariant {
        let annotation = serde_json::json!({
            "feature_id":"t".repeat(field_len), "genename":"g".repeat(field_len),
            "hgvs_c":"c".repeat(field_len), "hgvs_p":"p".repeat(field_len)
        });
        let hit: MyVariantHit = MyVariantHit::from_value(serde_json::json!({
            "_id":"x", "snpeff":{"ann":vec![annotation; annotations]}
        }))
        .unwrap();
        RetainedVariant {
            row: VariantSearchResult {
                id: format!("chr1:g.{id}A>G"),
                genome_build: super::super::GenomeBuild::Grch37,
                genome_build_provenance: "test".into(),
                gene: "SAFE".into(),
                hgvs_p: None,
                hgvs_c: None,
                transcript: None,
                legacy_name: None,
                significance: None,
                significance_source: None,
                significance_evaluated: None,
                clinvar_stars: None,
                gnomad_af: None,
                revel: None,
                gerp: None,
                source_identity: None,
                matched_alias: None,
                transcript_annotations_complete: None,
                transcript_annotations: None,
            },
            snpeff: hit.source().snpeff().cloned(),
            displayed_snpeff_index: None,
        }
    }

    let requested =
        RequestedVariantIdentity::for_search(Some("SAFE".into()), Some("A1V".into()), None, None);
    let exact = (0..8).map(|id| retained(id, 32, 256)).collect();
    let page = finalize_exact_page(&requested, exact, 0, 50, false, true);
    assert!(
        page.results
            .iter()
            .all(|row| row.transcript_annotations_complete == Some(true))
    );

    let mut over = (0..8).map(|id| retained(id, 32, 256)).collect::<Vec<_>>();
    let mut one_byte = retained(9, 0, 0);
    one_byte.snpeff = MyVariantHit::from_value(serde_json::json!({
        "_id":"x", "snpeff":{"ann":{"feature_id":"t"}}
    }))
    .unwrap()
    .source().snpeff().cloned();
    over.push(one_byte);
    let page = finalize_exact_page(&requested, over, 0, 50, false, true);
    assert!(page.results.iter().all(|row| {
        row.transcript_annotations_complete == Some(false)
            && row
                .transcript_annotations
                .as_deref()
                .is_some_and(<[VariantTranscriptAnnotation]>::is_empty)
    }));
}

#[test]
fn malformed_snpeff_isolated_from_exact_broad_and_get_siblings() {
    let make_hit = || -> MyVariantHit {
        crate::sources::myvariant::MyVariantHit::from_value(serde_json::json!({
            "_id":"chr5:g.118860951A>G",
            "dbnsfp":{"genename":"HSD17B4", "hgvsp":"p.His540Arg"},
            "clinvar":{"rcv":{"preferred_name":"NM_000414.3(HSD17B4):c.1544A>G (p.His515Arg)"}},
            "snpeff":{"ann":[{"feature_id":7}]}
        }))
        .unwrap()
    };
    let broad = transform::variant::from_myvariant_search_hit(&make_hit());
    assert_eq!(broad.transcript.as_deref(), Some("NM_000414.3"));
    let broad_json = serde_json::to_value(&broad).unwrap();
    assert!(broad_json.get("transcript_annotations").is_none());
    assert!(broad_json.get("transcript_annotations_complete").is_none());

    let detail = transform::variant::from_myvariant_hit(&make_hit());
    assert_eq!(detail.transcript.as_deref(), Some("NM_000414.3"));
    assert_eq!(detail.hgvs_c.as_deref(), Some("c.1544A>G"));
    assert_eq!(detail.hgvs_p.as_deref(), Some("p.His515Arg"));

    let no_valid_sibling: MyVariantHit = crate::sources::myvariant::MyVariantHit::from_value(serde_json::json!({
        "_id":"chr5:g.118860951A>G",
        "dbnsfp":{"genename":"HSD17B4", "hgvsc":["c.1544A>G", "c.1619A>G"], "hgvsp":["p.His515Arg", "p.His540Arg"]},
        "snpeff":{"ann":"malformed"}
    }))
    .unwrap();
    let no_sibling_broad = transform::variant::from_myvariant_search_hit(&no_valid_sibling);
    let no_sibling_detail = transform::variant::from_myvariant_hit(&no_valid_sibling);
    assert!(no_sibling_broad.transcript.is_none());
    assert!(no_sibling_broad.hgvs_c.is_none());
    assert!(no_sibling_broad.hgvs_p.is_none());
    assert!(no_sibling_detail.transcript.is_none());
    assert!(no_sibling_detail.hgvs_c.is_none());
    assert!(no_sibling_detail.hgvs_p.is_none());

    let requested = RequestedVariantIdentity::for_search(
        Some("HSD17B4".into()),
        Some("H540R".into()),
        None,
        None,
    );
    let mut seen = HashSet::new();
    let mut retained = Vec::new();
    retain_compatible_hits(&requested, [make_hit()], &mut seen, &mut retained);
    let exact = finalize_exact_page(&requested, retained, 0, 10, false, true);
    assert_eq!(
        exact.results[0].transcript_annotations_complete,
        Some(false)
    );
    assert!(
        exact.results[0]
            .transcript_annotations
            .as_deref()
            .is_some_and(<[VariantTranscriptAnnotation]>::is_empty)
    );
}

#[test]
fn incomplete_annotations_refuse_exact_metadata_and_keep_broad_get_siblings() {
    let malformed = [serde_json::json!({"ann":{"feature_id":7}})];
    let requested = RequestedVariantIdentity::for_search(
        Some("HSD17B4".into()),
        Some("H540R".into()),
        None,
        None,
    );
    for snpeff in malformed {
        let payload = serde_json::json!({
            "_id":"chr5:g.118860951A>G",
            "dbnsfp":{"genename":"HSD17B4", "hgvsp":"p.His540Arg"},
            "clinvar":{"rcv":{"preferred_name":"NM_000414.3(HSD17B4):c.1544A>G (p.His515Arg)"}},
            "snpeff":snpeff
        });
        let hit = || MyVariantHit::from_value(payload.clone()).unwrap();
        let broad = transform::variant::from_myvariant_search_hit(&hit());
        assert_eq!(broad.transcript.as_deref(), Some("NM_000414.3"));
        assert!(
            serde_json::to_value(&broad)
                .unwrap()
                .get("transcript_annotations")
                .is_none()
        );
        let detail = transform::variant::from_myvariant_hit(&hit());
        assert_eq!(detail.hgvs_p.as_deref(), Some("p.His515Arg"));
        let mut seen = HashSet::new();
        let mut retained = Vec::new();
        retain_compatible_hits(&requested, [hit()], &mut seen, &mut retained);
        let page = finalize_exact_page(&requested, retained, 0, 10, false, true);
        assert_eq!(page.results[0].transcript_annotations_complete, Some(false));
        assert!(
            page.results[0]
                .transcript_annotations
                .as_deref()
                .is_some_and(<[VariantTranscriptAnnotation]>::is_empty)
        );
    }
}

#[test]
fn absent_and_null_annotation_sets_are_exact_empty_complete() {
    for snpeff_member in [
        None,
        Some(serde_json::Value::Null),
        Some(serde_json::json!({})),
        Some(serde_json::json!({"ann":null})),
    ] {
        let mut payload = serde_json::json!({
            "_id":"chr5:g.118860951A>G",
            "dbnsfp":{"genename":"HSD17B4", "hgvsp":"p.His540Arg"}
        });
        if let Some(snpeff) = snpeff_member {
            payload["snpeff"] = snpeff;
        }
        let hit: MyVariantHit = MyVariantHit::from_value(payload).unwrap();
        let requested = RequestedVariantIdentity::for_search(
            Some("HSD17B4".into()),
            Some("H540R".into()),
            None,
            None,
        );
        let mut seen = HashSet::new();
        let mut retained = Vec::new();
        retain_compatible_hits(&requested, [hit], &mut seen, &mut retained);
        let page = finalize_exact_page(&requested, retained, 0, 10, false, true);
        assert_eq!(page.results[0].transcript_annotations_complete, Some(true));
        assert!(
            page.results[0]
                .transcript_annotations
                .as_deref()
                .is_some_and(<[VariantTranscriptAnnotation]>::is_empty)
        );
    }
}

fn refseq_request() -> RequestedVariantIdentity {
    RequestedVariantIdentity {
        gene: Some("ATM".into()),
        coding_change: Some("c.1066-6T>G".into()),
        transcript: Some("NM_000051.4".into()),
        genomic_accession: Some("NC_000011.10".into()),
        genome_build: Some("GRCh38".into()),
        position: Some(108248927),
        reference: Some("T".into()),
        alternate: Some("G".into()),
        ..Default::default()
    }
}

fn refseq_hit(id: &str, gene: Option<&str>, coding: Option<&str>) -> MyVariantHit {
    crate::sources::myvariant::MyVariantHit::from_value(serde_json::json!({
        "_id": id,
        "dbnsfp": {
            "genename": gene,
            "hgvsc": coding
        }
    }))
    .expect("valid MyVariant hit")
}

fn scan(
    requested: &RequestedVariantIdentity,
    hits: Vec<MyVariantHit>,
    exhaustive: bool,
) -> ProviderScan {
    ProviderScan {
        candidates: hits
            .into_iter()
            .map(|hit| provider_candidate(requested, hit))
            .collect(),
        exhaustive,
        available: true,
    }
}

#[test]
fn article_provider_aggregation_follows_precedence_and_refseq_fallback_states() {
    let requested = refseq_request();
    let compatible = refseq_hit(
        "GRCh38:NC_000011.10:g.108248927T>G",
        Some("ATM"),
        Some("NM_000051.4:c.1066-6T>G"),
    );
    let contradictory = refseq_hit(
        "GRCh38:NC_000011.10:g.108248928T>G",
        Some("ATM"),
        Some("NM_000051.4:c.1066-6T>G"),
    );

    let confirmed = article_resolution_context(
        requested.clone(),
        scan(
            &requested,
            vec![contradictory.clone(), compatible.clone()],
            true,
        ),
    );
    assert_eq!(
        confirmed.resolution.provider_validation.status,
        VariantProviderValidationStatus::Confirmed
    );
    assert_eq!(
        confirmed.resolution.basis,
        Some(VariantArticleResolutionBasis::ProviderConfirmed)
    );
    assert_eq!(
        confirmed.source_hit.as_ref().map(|hit| hit.source().id()),
        Some("GRCh38:NC_000011.10:g.108248927T>G")
    );

    let indeterminate = article_resolution_context(
        requested.clone(),
        scan(&requested, vec![compatible.clone()], false),
    );
    assert_eq!(
        indeterminate.resolution.provider_validation.status,
        VariantProviderValidationStatus::Indeterminate
    );
    assert_eq!(
        indeterminate.resolution.status,
        VariantResolutionStatus::Resolved
    );

    let contradictory = article_resolution_context(
        requested.clone(),
        scan(&requested, vec![contradictory], true),
    );
    assert_eq!(
        contradictory.resolution.provider_validation.status,
        VariantProviderValidationStatus::Contradictory
    );
    assert_eq!(
        contradictory
            .resolution
            .provider_validation
            .contradictory_field
            .as_deref(),
        Some("position")
    );
    assert_eq!(
        contradictory.resolution.status,
        VariantResolutionStatus::Unresolved
    );

    let not_found = article_resolution_context(requested.clone(), scan(&requested, vec![], true));
    assert_eq!(
        not_found.resolution.provider_validation.status,
        VariantProviderValidationStatus::NotFound
    );
    assert_eq!(
        not_found.resolution.basis,
        Some(VariantArticleResolutionBasis::CallerSupplied)
    );

    let unavailable = article_resolution_context(
        requested,
        ProviderScan {
            candidates: Vec::new(),
            exhaustive: false,
            available: false,
        },
    );
    assert_eq!(
        unavailable.resolution.provider_validation.status,
        VariantProviderValidationStatus::Unavailable
    );
    assert!(unavailable.available);
}

#[test]
fn article_provider_aggregation_marks_distinct_compatible_and_indeterminate_sets() {
    let requested = refseq_request();
    let compatible = refseq_hit(
        "GRCh38:NC_000011.10:g.108248927T>G",
        Some("ATM"),
        Some("NM_000051.4:c.1066-6T>G"),
    );
    let compatible_with_rsid: MyVariantHit = crate::sources::myvariant::MyVariantHit::from_value(serde_json::json!({
        "_id": "GRCh38:NC_000011.10:g.108248927T>G",
        "dbnsfp": {
            "genename": "ATM",
            "hgvsc": "NM_000051.4:c.1066-6T>G"
        },
        "dbsnp": {"rsid": "rs605"}
    }))
    .expect("valid MyVariant hit");
    let multiple = article_resolution_context(
        requested.clone(),
        scan(
            &requested,
            vec![compatible.clone(), compatible_with_rsid],
            true,
        ),
    );
    assert_eq!(
        multiple.resolution.provider_validation.status,
        VariantProviderValidationStatus::Indeterminate
    );

    let missing_facts = refseq_hit("GRCh38:NC_000011.10:g.108248927T>G", None, None);
    let with_indeterminate = article_resolution_context(
        requested.clone(),
        scan(&requested, vec![compatible, missing_facts], true),
    );
    assert_eq!(
        with_indeterminate.resolution.provider_validation.status,
        VariantProviderValidationStatus::Indeterminate
    );

    let position_first = refseq_hit(
        "GRCh38:NC_000011.10:g.108248926T>G",
        Some("ATM"),
        Some("NM_000051.4:c.1066-6T>G"),
    );
    let gene_second = refseq_hit(
        "GRCh38:NC_000011.10:g.108248928T>G",
        Some("OTHER"),
        Some("NM_000051.4:c.1066-6T>G"),
    );
    let left = article_resolution_context(
        requested.clone(),
        scan(
            &requested,
            vec![gene_second.clone(), position_first.clone()],
            true,
        ),
    );
    let right = article_resolution_context(
        requested.clone(),
        scan(&requested, vec![position_first, gene_second], true),
    );
    assert_eq!(left.resolution, right.resolution);
    assert_eq!(
        left.resolution
            .provider_validation
            .contradictory_field
            .as_deref(),
        Some("position")
    );
}

#[test]
fn article_provider_aggregation_is_order_independent_and_selects_stable_alias() {
    let requested = RequestedVariantIdentity {
        coding_change: None,
        transcript: None,
        ..refseq_request()
    };
    let first = refseq_hit(
        "GRCh38:NC_000011.10:g.108248927T>G",
        Some("ATM"),
        Some("c.1066-6T>G"),
    );
    let second = refseq_hit(
        "GRCh38:NC_000011.10:g.108248927T>G",
        Some("ATM"),
        Some("NM_000051.4:c.1066-6T>G"),
    );
    let left = article_resolution_context(
        requested.clone(),
        scan(&requested, vec![first.clone(), second.clone()], true),
    );
    let right = article_resolution_context(
        requested.clone(),
        scan(&requested, vec![second, first], true),
    );
    assert_eq!(left.resolution, right.resolution);
    assert_eq!(
        left.resolution.provider_validation.matched_alias.as_deref(),
        Some("GRCh38:NC_000011.10:g.108248927T>G")
    );
}
