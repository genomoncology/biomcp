use super::*;

#[test]
fn chart_args_default_to_no_chart() {
    let args = ChartArgs {
        chart: None,
        terminal: false,
        output: None,
        title: None,
        theme: None,
        palette: None,
        cols: None,
        rows: None,
        width: None,
        height: None,
        scale: None,
        mcp_inline: false,
    };
    assert_eq!(args.chart, None);
    assert!(!args.terminal);
    assert!(!args.mcp_inline);
    assert_eq!(args.cols, None);
    assert_eq!(args.rows, None);
    assert_eq!(args.width, None);
    assert_eq!(args.height, None);
    assert_eq!(args.scale, None);
}

#[test]
fn chart_dimension_flags_validate_positive_values() {
    let cols_err = Cli::try_parse_from([
        "biomcp",
        "study",
        "query",
        "--study",
        "msk_impact_2017",
        "--gene",
        "TP53",
        "--type",
        "mutations",
        "--chart",
        "bar",
        "--cols",
        "0",
    ])
    .expect_err("zero columns should fail");
    assert!(cols_err.to_string().contains("--cols must be >= 1"));

    let scale_err = Cli::try_parse_from([
        "biomcp",
        "study",
        "query",
        "--study",
        "msk_impact_2017",
        "--gene",
        "TP53",
        "--type",
        "mutations",
        "--chart",
        "bar",
        "--scale",
        "0",
    ])
    .expect_err("zero scale should fail");
    assert!(scale_err.to_string().contains("--scale must be > 0"));

    let nan_err = Cli::try_parse_from([
        "biomcp",
        "study",
        "query",
        "--study",
        "msk_impact_2017",
        "--gene",
        "TP53",
        "--type",
        "mutations",
        "--chart",
        "bar",
        "--scale",
        "NaN",
        "-o",
        "chart.png",
    ])
    .expect_err("non-finite scale should fail");
    assert!(
        nan_err
            .to_string()
            .contains("--scale must be a finite number > 0")
    );
}
