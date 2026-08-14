use super::*;

#[tokio::test]
async fn study_co_occurrence_requires_2_to_10_genes() {
    let err = execute(vec![
        "biomcp".to_string(),
        "study".to_string(),
        "co-occurrence".to_string(),
        "--study".to_string(),
        "msk_impact_2017".to_string(),
        "--genes".to_string(),
        "TP53".to_string(),
    ])
    .await
    .expect_err("study co-occurrence should validate gene count");
    assert!(err.to_string().contains("--genes must contain 2 to 10"));
}

#[tokio::test]
async fn study_filter_requires_at_least_one_criterion() {
    let err = execute(vec![
        "biomcp".to_string(),
        "study".to_string(),
        "filter".to_string(),
        "--study".to_string(),
        "brca_tcga_pan_can_atlas_2018".to_string(),
    ])
    .await
    .expect_err("study filter should require criteria");
    assert!(
        err.to_string()
            .contains("At least one filter criterion is required")
    );
}

#[tokio::test]
async fn study_filter_rejects_malformed_expression_threshold() {
    let err = execute(vec![
        "biomcp".to_string(),
        "study".to_string(),
        "filter".to_string(),
        "--study".to_string(),
        "brca_tcga_pan_can_atlas_2018".to_string(),
        "--expression-above".to_string(),
        "MYC:not-a-number".to_string(),
    ])
    .await
    .expect_err("study filter should validate threshold format");
    assert!(err.to_string().contains("--expression-above"));
    assert!(err.to_string().contains("GENE:THRESHOLD"));
}

#[tokio::test]
async fn study_filter_rejects_non_finite_expression_thresholds_before_study_lookup() {
    for (flag, threshold) in [
        ("--expression-above", "NaN"),
        ("--expression-above", "inf"),
        ("--expression-above", "-inf"),
        ("--expression-above", "1e309"),
        ("--expression-below", "NaN"),
        ("--expression-below", "inf"),
        ("--expression-below", "-inf"),
        ("--expression-below", "-1e309"),
    ] {
        let value = format!("MYC:{threshold}");
        let err = execute(vec![
            "biomcp".to_string(),
            "study".to_string(),
            "filter".to_string(),
            "--study".to_string(),
            "definitely-not-a-local-study".to_string(),
            flag.to_string(),
            value.clone(),
        ])
        .await
        .expect_err("non-finite thresholds must fail before study lookup");
        let message = err.to_string();
        assert!(message.contains(flag), "message={message}");
        assert!(message.contains(&value), "message={message}");
        assert!(message.contains("finite"), "message={message}");
        assert!(
            !message.contains("not found") && !message.contains("not available locally"),
            "validation reached study lookup: {message}"
        );
    }
}

#[tokio::test]
async fn study_filter_cli_and_mcp_paths_share_non_finite_invalid_argument() {
    let args = vec![
        "biomcp".to_string(),
        "study".to_string(),
        "filter".to_string(),
        "--study".to_string(),
        "definitely-not-a-local-study".to_string(),
        "--expression-above".to_string(),
        "MYC:NaN".to_string(),
    ];

    let cli_error = execute(args.clone())
        .await
        .expect_err("CLI rejects a non-finite threshold");
    let mcp_error = crate::cli::execute_mcp(args)
        .await
        .expect_err("raw MCP rejects the same non-finite threshold");

    assert!(
        cli_error
            .downcast_ref::<crate::error::BioMcpError>()
            .is_some()
    );
    assert!(
        mcp_error
            .downcast_ref::<crate::error::BioMcpError>()
            .is_some()
    );
    assert_eq!(cli_error.to_string(), mcp_error.to_string());
    assert!(cli_error.to_string().contains("Invalid argument"));
}

#[test]
fn study_filter_accepts_finite_f64_threshold_boundaries() {
    let cases = [
        ("0", 0.0),
        ("-0", -0.0),
        ("1.7976931348623157e308", f64::MAX),
        ("-1.7976931348623157e308", f64::MIN),
        ("2.2250738585072014e-308", f64::MIN_POSITIVE),
        ("5e-324", f64::from_bits(1)),
    ];

    for (text, expected) in cases {
        let parsed = super::super::dispatch::parse_expression_filter(
            &format!("MYC:{text}"),
            "--expression-above",
            crate::entities::study::FilterCriterion::ExpressionAbove,
        )
        .expect("every finite Rust f64 spelling remains accepted");
        let crate::entities::study::FilterCriterion::ExpressionAbove(gene, threshold) = parsed
        else {
            panic!("unexpected criterion")
        };
        assert_eq!(gene, "MYC");
        assert_eq!(threshold.to_bits(), expected.to_bits(), "threshold={text}");
    }
}

#[tokio::test]
async fn study_survival_rejects_unknown_endpoint() {
    let err = execute(vec![
        "biomcp".to_string(),
        "study".to_string(),
        "survival".to_string(),
        "--study".to_string(),
        "msk_impact_2017".to_string(),
        "--gene".to_string(),
        "TP53".to_string(),
        "--endpoint".to_string(),
        "foo".to_string(),
    ])
    .await
    .expect_err("study survival should validate endpoint");
    assert!(err.to_string().contains("Unknown survival endpoint"));
}

#[tokio::test]
async fn study_compare_rejects_unknown_type() {
    let err = execute(vec![
        "biomcp".to_string(),
        "study".to_string(),
        "compare".to_string(),
        "--study".to_string(),
        "msk_impact_2017".to_string(),
        "--gene".to_string(),
        "TP53".to_string(),
        "--type".to_string(),
        "foo".to_string(),
        "--target".to_string(),
        "ERBB2".to_string(),
    ])
    .await
    .expect_err("study compare should validate type");
    assert!(err.to_string().contains("Unknown comparison type"));
}
