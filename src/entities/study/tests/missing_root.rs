use super::*;

#[test]
fn normalize_study_id_rejects_path_like_input() {
    let err = normalize_study_id("../demo_study").expect_err("path-like study ID should fail");
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("Invalid study ID"));
}

#[tokio::test]
async fn missing_study_root_is_an_empty_catalog() {
    let fixture = TestStudyDir::new("missing-list-root");
    let missing = fixture.root.join("not-created");

    let studies = list_studies_with_root(missing.clone())
        .await
        .expect("missing study root should be empty");

    assert!(studies.is_empty());
    assert!(!missing.exists());
}

#[tokio::test]
async fn query_with_a_missing_root_returns_not_in_local_cohorts() {
    let fixture = TestStudyDir::new("missing-query-root");
    let missing = fixture.root.join("not-created");

    let result = query_study_with_root(
        missing.clone(),
        "msk_impact_2017",
        "TP53",
        StudyQueryType::Mutations,
    )
    .await
    .expect("missing root should report local coverage");

    let StudyQueryResult::NotInLocalCohorts(result) = result else {
        panic!("expected not_in_local_cohorts");
    };
    assert!(result.local_study_ids.is_empty());
    assert_eq!(
        result.coverage_status,
        LocalCohortCoverageStatus::NotInLocalCohorts
    );
    assert!(!missing.exists());
}
