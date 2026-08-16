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
fn chart_dimension_flags_enforce_practical_bounds() {
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
        "19",
    ])
    .expect_err("too few columns should fail");
    assert!(
        cols_err
            .to_string()
            .contains("--cols must be between 20 and 500")
    );

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
        "0.49",
    ])
    .expect_err("too-small scale should fail");
    assert!(
        scale_err
            .to_string()
            .contains("--scale must be between 0.5 and 4")
    );

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
            .contains("--scale must be a finite number between 0.5 and 4")
    );

    for (flag, value, message) in [
        ("--cols", "501", "--cols must be between 20 and 500"),
        ("--rows", "7", "--rows must be between 8 and 200"),
        ("--rows", "201", "--rows must be between 8 and 200"),
        ("--width", "239", "--width must be between 240 and 4096"),
        ("--width", "4097", "--width must be between 240 and 4096"),
        ("--height", "159", "--height must be between 160 and 4096"),
        ("--height", "4097", "--height must be between 160 and 4096"),
        ("--scale", "4.01", "--scale must be between 0.5 and 4"),
    ] {
        let err = Cli::try_parse_from([
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
            flag,
            value,
        ])
        .expect_err("out-of-range chart dimension should fail");
        assert!(err.to_string().contains(message), "{flag}={value}: {err}");
    }
}

#[tokio::test]
async fn chart_json_uses_the_canonical_name_for_every_topic() {
    for name in [
        "bar",
        "stacked-bar",
        "pie",
        "waterfall",
        "heatmap",
        "histogram",
        "density",
        "box",
        "violin",
        "ridgeline",
        "scatter",
        "survival",
    ] {
        let output = execute(vec![
            "biomcp".into(),
            "--json".into(),
            "chart".into(),
            name.into(),
        ])
        .await
        .expect("chart topic should render");
        let value: serde_json::Value =
            serde_json::from_str(&output).expect("chart output should be JSON");
        assert_eq!(value["chart"], name);
    }
}
