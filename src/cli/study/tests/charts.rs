use super::*;

#[test]
fn study_query_parses_chart_flags() {
    let cli = Cli::try_parse_from([
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
        "--terminal",
        "--cols",
        "80",
        "--rows",
        "24",
        "--title",
        "TP53 mutations",
        "--theme",
        "dark",
        "--palette",
        "wong",
    ])
    .expect("study query chart flags should parse");

    let Cli {
        command: Commands::Study {
            cmd: StudyCommand::Query { chart, .. },
        },
        ..
    } = cli
    else {
        panic!("expected study query command");
    };

    assert_eq!(chart.chart, Some(ChartType::Bar));
    assert!(chart.terminal);
    assert_eq!(chart.cols, Some(80));
    assert_eq!(chart.rows, Some(24));
    assert_eq!(chart.title.as_deref(), Some("TP53 mutations"));
    assert_eq!(chart.theme.as_deref(), Some("dark"));
    assert_eq!(chart.palette.as_deref(), Some("wong"));
}

#[test]
fn study_query_parses_waterfall_chart_flag() {
    let cli = Cli::try_parse_from([
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
        "waterfall",
        "--terminal",
    ])
    .expect("study query waterfall chart should parse");

    let Cli {
        command: Commands::Study {
            cmd: StudyCommand::Query { chart, .. },
        },
        ..
    } = cli
    else {
        panic!("expected study query command");
    };

    assert_eq!(chart.chart, Some(ChartType::Waterfall));
    assert!(chart.terminal);
}

#[test]
fn study_co_occurrence_parses_heatmap_chart_flag() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "study",
        "co-occurrence",
        "--study",
        "brca_tcga_pan_can_atlas_2018",
        "--genes",
        "TP53,PIK3CA,GATA3",
        "--chart",
        "heatmap",
        "--terminal",
    ])
    .expect("study co-occurrence heatmap chart should parse");

    let Cli {
        command:
            Commands::Study {
                cmd: StudyCommand::CoOccurrence { chart, .. },
            },
        ..
    } = cli
    else {
        panic!("expected study co-occurrence command");
    };

    assert_eq!(chart.chart, Some(ChartType::Heatmap));
    assert!(chart.terminal);
}

#[test]
fn study_compare_mutations_parses_stacked_bar_chart_flag() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "study",
        "compare",
        "--study",
        "brca_tcga_pan_can_atlas_2018",
        "--gene",
        "TP53",
        "--type",
        "mutations",
        "--target",
        "PIK3CA",
        "--chart",
        "stacked-bar",
        "--terminal",
    ])
    .expect("study compare stacked-bar chart should parse");

    let Cli {
        command: Commands::Study {
            cmd: StudyCommand::Compare { chart, .. },
        },
        ..
    } = cli
    else {
        panic!("expected study compare command");
    };

    assert_eq!(chart.chart, Some(ChartType::StackedBar));
    assert!(chart.terminal);
}

#[test]
fn study_compare_expression_parses_scatter_chart_with_file_dimensions() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "study",
        "compare",
        "--study",
        "brca_tcga_pan_can_atlas_2018",
        "--gene",
        "TP53",
        "--type",
        "expression",
        "--target",
        "ERBB2",
        "--chart",
        "scatter",
        "--width",
        "1200",
        "--height",
        "600",
        "-o",
        "scatter.svg",
    ])
    .expect("study compare scatter chart should parse");

    let Cli {
        command: Commands::Study {
            cmd: StudyCommand::Compare { chart, .. },
        },
        ..
    } = cli
    else {
        panic!("expected study compare command");
    };

    assert_eq!(chart.chart, Some(ChartType::Scatter));
    assert_eq!(chart.width, Some(1200));
    assert_eq!(chart.height, Some(600));
    assert_eq!(
        chart.output.as_deref(),
        Some(std::path::Path::new("scatter.svg"))
    );
}

#[test]
fn study_survival_parses_survival_chart_flag() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "study",
        "survival",
        "--study",
        "brca_tcga_pan_can_atlas_2018",
        "--gene",
        "TP53",
        "--chart",
        "survival",
        "--terminal",
    ])
    .expect("study survival chart flags should parse");

    let Cli {
        command: Commands::Study {
            cmd: StudyCommand::Survival { chart, .. },
        },
        ..
    } = cli
    else {
        panic!("expected study survival command");
    };

    assert_eq!(chart.chart, Some(ChartType::Survival));
    assert!(chart.terminal);
}

#[tokio::test]
async fn png_pixel_budget_is_rejected_before_study_lookup() {
    let err = execute(vec![
        "biomcp".into(),
        "study".into(),
        "query".into(),
        "--study".into(),
        "definitely_missing".into(),
        "--gene".into(),
        "TP53".into(),
        "--type".into(),
        "mutations".into(),
        "--chart".into(),
        "bar".into(),
        "--width".into(),
        "4096".into(),
        "--height".into(),
        "4096".into(),
        "--scale".into(),
        "2".into(),
        "--output".into(),
        "oversized.png".into(),
    ])
    .await
    .expect_err("oversized PNG should fail before study lookup");

    assert!(
        err.to_string()
            .contains("PNG output is limited to 16777216 pixels")
    );
    assert!(!std::path::Path::new("oversized.png").exists());
}

#[test]
fn chart_auxiliary_flags_require_chart() {
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
        "--terminal",
    ])
    .expect_err("--terminal without --chart should fail");

    assert!(err.to_string().contains("--chart"));
}

#[test]
fn short_help_hides_chart_flags_but_long_help_shows_them() {
    let mut command = Cli::command();
    let study = command
        .find_subcommand_mut("study")
        .expect("study subcommand should exist");
    let query = study
        .find_subcommand_mut("query")
        .expect("study query subcommand should exist");

    let mut short_help = Vec::new();
    query
        .write_help(&mut short_help)
        .expect("short help should render");
    let short_help = String::from_utf8(short_help).expect("short help should be utf-8");
    assert!(!short_help.contains("--theme"));
    assert!(!short_help.contains("--palette"));

    let mut long_help = Vec::new();
    query
        .write_long_help(&mut long_help)
        .expect("long help should render");
    let long_help = String::from_utf8(long_help).expect("long help should be utf-8");
    assert!(long_help.contains("--theme <THEME>"));
    assert!(long_help.contains("--palette <PALETTE>"));
}

#[tokio::test]
async fn handle_command_rejects_invalid_expression_chart() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "study",
        "compare",
        "--study",
        "msk_impact_2017",
        "--gene",
        "TP53",
        "--type",
        "expression",
        "--target",
        "ERBB2",
        "--chart",
        "pie",
        "--terminal",
    ])
    .expect("study compare should parse");

    let Cli {
        command: Commands::Study { cmd },
        json,
        ..
    } = cli
    else {
        panic!("expected study compare command");
    };

    let err = handle_command(cmd, json)
        .await
        .expect_err("expression compare should reject pie");
    let msg = err.to_string();
    assert!(msg.contains("study compare --type expression"));
    assert!(msg.contains("box"));
    assert!(msg.contains("violin"));
    assert!(msg.contains("ridgeline"));
    assert!(msg.contains("scatter"));
}

#[tokio::test]
async fn study_co_occurrence_invalid_chart_lists_heatmap() {
    let err = execute(vec![
        "biomcp".to_string(),
        "study".to_string(),
        "co-occurrence".to_string(),
        "--study".to_string(),
        "msk_impact_2017".to_string(),
        "--genes".to_string(),
        "TP53,KRAS".to_string(),
        "--chart".to_string(),
        "violin".to_string(),
        "--terminal".to_string(),
    ])
    .await
    .expect_err("study co-occurrence should reject violin");
    let msg = err.to_string();
    assert!(msg.contains("study co-occurrence"));
    assert!(msg.contains("bar"));
    assert!(msg.contains("pie"));
    assert!(msg.contains("heatmap"));
}

#[tokio::test]
async fn study_query_mutations_invalid_chart_lists_waterfall() {
    let err = execute(vec![
        "biomcp".to_string(),
        "study".to_string(),
        "query".to_string(),
        "--study".to_string(),
        "msk_impact_2017".to_string(),
        "--gene".to_string(),
        "TP53".to_string(),
        "--type".to_string(),
        "mutations".to_string(),
        "--chart".to_string(),
        "violin".to_string(),
        "--terminal".to_string(),
    ])
    .await
    .expect_err("study query mutations should reject violin");
    let msg = err.to_string();
    assert!(msg.contains("study query --type mutations"));
    assert!(msg.contains("bar"));
    assert!(msg.contains("pie"));
    assert!(msg.contains("waterfall"));
}

#[tokio::test]
async fn study_compare_mutations_invalid_chart_lists_stacked_bar() {
    let err = execute(vec![
        "biomcp".to_string(),
        "study".to_string(),
        "compare".to_string(),
        "--study".to_string(),
        "msk_impact_2017".to_string(),
        "--gene".to_string(),
        "TP53".to_string(),
        "--type".to_string(),
        "mutations".to_string(),
        "--target".to_string(),
        "KRAS".to_string(),
        "--chart".to_string(),
        "violin".to_string(),
        "--terminal".to_string(),
    ])
    .await
    .expect_err("mutation compare should reject violin");
    let msg = err.to_string();
    assert!(msg.contains("study compare --type mutations"));
    assert!(msg.contains("bar"));
    assert!(msg.contains("stacked-bar"));
}
