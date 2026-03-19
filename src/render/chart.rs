use std::fs;
use std::path::PathBuf;

use kuva::backend::svg::SvgBackend;
use kuva::backend::terminal::TerminalBackend;
use kuva::plot::{BarPlot, BoxPlot, DensityPlot, Histogram, PiePlot, RidgelinePlot, ViolinPlot};
use kuva::prelude::{Layout, Palette, PieLabelPosition, Plot, Theme, render_multiple};

#[cfg(feature = "charts-png")]
use kuva::PngBackend;

use crate::cli::ChartType;
use crate::entities::study::{
    CnaDistributionResult, CoOccurrenceResult, MutationComparisonResult, MutationFrequencyResult,
    StudyQueryType, SurvivalResult,
};
use crate::error::BioMcpError;

const TERMINAL_COLS: usize = 100;
const TERMINAL_ROWS: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChartRenderOptions {
    pub terminal: bool,
    pub inline_svg: bool,
    pub output: Option<PathBuf>,
    pub title: Option<String>,
    pub theme: Option<String>,
    pub palette: Option<String>,
}

impl ChartRenderOptions {
    pub(crate) fn from_args(
        terminal: bool,
        inline_svg: bool,
        output: Option<PathBuf>,
        title: Option<String>,
        theme: Option<String>,
        palette: Option<String>,
    ) -> Self {
        Self {
            terminal,
            inline_svg,
            output,
            title,
            theme,
            palette,
        }
    }
}

enum OutputTarget {
    Terminal,
    Svg(PathBuf),
    Png(PathBuf),
    InlineSvg,
}

pub(crate) fn validate_query_chart_type(
    query_type: StudyQueryType,
    chart_type: ChartType,
) -> Result<(), BioMcpError> {
    match query_type {
        StudyQueryType::Mutations => validate_standalone_chart_type(
            "study query --type mutations",
            chart_type,
            &[ChartType::Bar, ChartType::Pie],
        ),
        StudyQueryType::Cna => validate_standalone_chart_type(
            "study query --type cna",
            chart_type,
            &[ChartType::Bar, ChartType::Pie],
        ),
        StudyQueryType::Expression => validate_standalone_chart_type(
            "study query --type expression",
            chart_type,
            &[ChartType::Histogram, ChartType::Density],
        ),
    }
}

pub(crate) fn validate_compare_chart_type(
    compare_type: &str,
    chart_type: ChartType,
) -> Result<(), BioMcpError> {
    match compare_type.trim().to_ascii_lowercase().as_str() {
        "expression" | "expr" => validate_standalone_chart_type(
            "study compare --type expression",
            chart_type,
            &[ChartType::Box, ChartType::Violin, ChartType::Ridgeline],
        ),
        "mutations" | "mutation" => validate_standalone_chart_type(
            "study compare --type mutations",
            chart_type,
            &[ChartType::Bar],
        ),
        other => Err(BioMcpError::InvalidArgument(format!(
            "Unknown comparison type '{other}'. Expected: expression, mutations."
        ))),
    }
}

pub(crate) fn validate_standalone_chart_type(
    command_label: &str,
    chart_type: ChartType,
    valid_types: &[ChartType],
) -> Result<(), BioMcpError> {
    if valid_types.contains(&chart_type) {
        return Ok(());
    }
    Err(BioMcpError::InvalidArgument(format!(
        "chart type '{chart_type}' is not valid for '{command_label}'. Valid types: {}",
        valid_types
            .iter()
            .map(|value| value.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    )))
}

pub(crate) fn render_mutation_frequency_chart(
    result: &MutationFrequencyResult,
    chart_type: ChartType,
    options: &ChartRenderOptions,
) -> Result<String, BioMcpError> {
    validate_standalone_chart_type(
        "study query --type mutations",
        chart_type,
        &[ChartType::Bar, ChartType::Pie],
    )?;
    let palette = palette_colors(options.palette.as_deref())?;
    let title = format!("{} mutation classes", result.gene);
    match chart_type {
        ChartType::Bar => {
            let bars = if result.top_variant_classes.is_empty() {
                vec![
                    ("Mutated".to_string(), result.unique_samples as f64),
                    (
                        "Wildtype".to_string(),
                        result.total_samples.saturating_sub(result.unique_samples) as f64,
                    ),
                ]
            } else {
                result
                    .top_variant_classes
                    .iter()
                    .map(|(label, count)| (label.clone(), *count as f64))
                    .collect()
            };
            let plot = BarPlot::new()
                .with_bars(bars)
                .with_color(palette[0].clone());
            render_chart(
                vec![Plot::Bar(plot)],
                options,
                &title,
                "Variant class",
                "Count",
            )
        }
        ChartType::Pie => {
            let slices = if result.top_variant_classes.is_empty() {
                vec![
                    ("Mutated".to_string(), result.unique_samples as f64),
                    (
                        "Wildtype".to_string(),
                        result.total_samples.saturating_sub(result.unique_samples) as f64,
                    ),
                ]
            } else {
                result
                    .top_variant_classes
                    .iter()
                    .map(|(label, count)| (label.clone(), *count as f64))
                    .collect()
            };
            let mut plot = PiePlot::new()
                .with_legend("Variant class")
                .with_percent()
                .with_label_position(PieLabelPosition::Auto);
            for (idx, (label, value)) in slices.into_iter().enumerate() {
                plot = plot.with_slice(label, value, palette[idx % palette.len()].clone());
            }
            render_chart(vec![Plot::Pie(plot)], options, &title, "Class", "Count")
        }
        other => Err(BioMcpError::InvalidArgument(format!(
            "Unsupported mutation chart type '{other}'"
        ))),
    }
}

pub(crate) fn render_cna_chart(
    result: &CnaDistributionResult,
    chart_type: ChartType,
    options: &ChartRenderOptions,
) -> Result<String, BioMcpError> {
    validate_standalone_chart_type(
        "study query --type cna",
        chart_type,
        &[ChartType::Bar, ChartType::Pie],
    )?;
    let palette = palette_colors(options.palette.as_deref())?;
    let categories = vec![
        (
            "Deep deletion (-2)".to_string(),
            result.deep_deletion as f64,
        ),
        (
            "Shallow deletion (-1)".to_string(),
            result.shallow_deletion as f64,
        ),
        ("Diploid (0)".to_string(), result.diploid as f64),
        ("Gain (1)".to_string(), result.gain as f64),
        ("Amplification (2)".to_string(), result.amplification as f64),
    ];
    let title = format!("{} CNA distribution", result.gene);
    match chart_type {
        ChartType::Bar => {
            let plot = BarPlot::new()
                .with_bars(categories.clone())
                .with_color(palette[0].clone());
            render_chart(vec![Plot::Bar(plot)], options, &title, "Bucket", "Count")
        }
        ChartType::Pie => {
            let mut plot = PiePlot::new()
                .with_legend("CNA bucket")
                .with_percent()
                .with_label_position(PieLabelPosition::Auto);
            for (idx, (label, value)) in categories.into_iter().enumerate() {
                plot = plot.with_slice(label, value, palette[idx % palette.len()].clone());
            }
            render_chart(vec![Plot::Pie(plot)], options, &title, "Bucket", "Count")
        }
        other => Err(BioMcpError::InvalidArgument(format!(
            "Unsupported CNA chart type '{other}'"
        ))),
    }
}

pub(crate) fn render_expression_histogram_chart(
    study_id: &str,
    gene: &str,
    values: &[f64],
    options: &ChartRenderOptions,
) -> Result<String, BioMcpError> {
    let palette = palette_colors(options.palette.as_deref())?;
    let range = numeric_range(values)?;
    let plot = Histogram::new()
        .with_data(values.iter().copied())
        .with_bins(suggest_bins(values.len()))
        .with_range(range)
        .with_color(palette[0].clone());
    render_chart(
        vec![Plot::Histogram(plot)],
        options,
        &format!("{gene} expression histogram ({study_id})"),
        "Expression",
        "Count",
    )
}

pub(crate) fn render_expression_density_chart(
    study_id: &str,
    gene: &str,
    values: &[f64],
    options: &ChartRenderOptions,
) -> Result<String, BioMcpError> {
    ensure_non_empty(values, "expression density")?;
    let palette = palette_colors(options.palette.as_deref())?;
    let plot = DensityPlot::new()
        .with_data(values.iter().copied())
        .with_color(palette[0].clone())
        .with_filled(true);
    render_chart(
        vec![Plot::Density(plot)],
        options,
        &format!("{gene} expression density ({study_id})"),
        "Expression",
        "Density",
    )
}

pub(crate) fn render_expression_compare_chart(
    study_id: &str,
    stratify_gene: &str,
    target_gene: &str,
    groups: &[(String, Vec<f64>)],
    chart_type: ChartType,
    options: &ChartRenderOptions,
) -> Result<String, BioMcpError> {
    validate_standalone_chart_type(
        "study compare --type expression",
        chart_type,
        &[ChartType::Box, ChartType::Violin, ChartType::Ridgeline],
    )?;
    if groups.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Expression comparison chart requires at least one group.".into(),
        ));
    }
    let palette = palette_colors(options.palette.as_deref())?;
    let title = format!("{target_gene} by {stratify_gene} status ({study_id})");
    match chart_type {
        ChartType::Box => {
            let mut plot = BoxPlot::new();
            for (label, values) in groups {
                plot = plot.with_group(label.clone(), values.iter().copied());
            }
            plot = plot.with_group_colors(
                palette
                    .iter()
                    .take(groups.len())
                    .cloned()
                    .collect::<Vec<_>>(),
            );
            render_chart(
                vec![Plot::Box(plot)],
                options,
                &title,
                "Group",
                "Expression",
            )
        }
        ChartType::Violin => {
            let mut plot = ViolinPlot::new();
            for (label, values) in groups {
                plot = plot.with_group(label.clone(), values.iter().copied());
            }
            plot = plot.with_group_colors(
                palette
                    .iter()
                    .take(groups.len())
                    .cloned()
                    .collect::<Vec<_>>(),
            );
            render_chart(
                vec![Plot::Violin(plot)],
                options,
                &title,
                "Group",
                "Expression",
            )
        }
        ChartType::Ridgeline => {
            let mut plot = RidgelinePlot::new().with_legend(true);
            for (idx, (label, values)) in groups.iter().enumerate() {
                plot = plot.with_group_color(
                    label.clone(),
                    values.iter().copied(),
                    palette[idx % palette.len()].clone(),
                );
            }
            render_chart(
                vec![Plot::Ridgeline(plot)],
                options,
                &title,
                "Expression",
                "Group",
            )
        }
        other => Err(BioMcpError::InvalidArgument(format!(
            "Unsupported expression comparison chart type '{other}'"
        ))),
    }
}

pub(crate) fn render_mutation_compare_chart(
    result: &MutationComparisonResult,
    chart_type: ChartType,
    options: &ChartRenderOptions,
) -> Result<String, BioMcpError> {
    validate_standalone_chart_type(
        "study compare --type mutations",
        chart_type,
        &[ChartType::Bar],
    )?;
    let palette = palette_colors(options.palette.as_deref())?;
    let plot = BarPlot::new()
        .with_bars(
            result
                .groups
                .iter()
                .map(|group| (group.group_name.clone(), group.mutation_rate))
                .collect::<Vec<_>>(),
        )
        .with_color(palette[0].clone());
    render_chart(
        vec![Plot::Bar(plot)],
        options,
        &format!(
            "{} mutation rate by {}",
            result.target_gene, result.stratify_gene
        ),
        "Group",
        "Mutation rate",
    )
}

pub(crate) fn render_co_occurrence_chart(
    result: &CoOccurrenceResult,
    chart_type: ChartType,
    options: &ChartRenderOptions,
) -> Result<String, BioMcpError> {
    validate_standalone_chart_type(
        "study co-occurrence",
        chart_type,
        &[ChartType::Bar, ChartType::Pie],
    )?;
    if result.pairs.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Co-occurrence chart requires at least one gene pair.".into(),
        ));
    }
    let palette = palette_colors(options.palette.as_deref())?;
    match chart_type {
        ChartType::Bar => {
            let plot = BarPlot::new()
                .with_bars(
                    result
                        .pairs
                        .iter()
                        .map(|pair| {
                            (
                                format!("{}/{}", pair.gene_a, pair.gene_b),
                                pair.both_mutated as f64,
                            )
                        })
                        .collect::<Vec<_>>(),
                )
                .with_color(palette[0].clone());
            render_chart(
                vec![Plot::Bar(plot)],
                options,
                &format!("Co-occurrence in {}", result.study_id),
                "Gene pair",
                "Both mutated",
            )
        }
        ChartType::Pie => {
            let pair = &result.pairs[0];
            let mut plot = PiePlot::new()
                .with_legend("Contingency")
                .with_percent()
                .with_label_position(PieLabelPosition::Auto);
            for (idx, (label, value)) in [
                ("Both mutated".to_string(), pair.both_mutated as f64),
                (format!("{} only", pair.gene_a), pair.a_only as f64),
                (format!("{} only", pair.gene_b), pair.b_only as f64),
                ("Neither".to_string(), pair.neither as f64),
            ]
            .into_iter()
            .enumerate()
            {
                plot = plot.with_slice(label, value, palette[idx % palette.len()].clone());
            }
            render_chart(
                vec![Plot::Pie(plot)],
                options,
                &format!("{} / {} contingency", pair.gene_a, pair.gene_b),
                "Category",
                "Count",
            )
        }
        other => Err(BioMcpError::InvalidArgument(format!(
            "Unsupported co-occurrence chart type '{other}'"
        ))),
    }
}

pub(crate) fn render_survival_chart(
    result: &SurvivalResult,
    chart_type: ChartType,
    options: &ChartRenderOptions,
) -> Result<String, BioMcpError> {
    validate_standalone_chart_type("study survival", chart_type, &[ChartType::Bar])?;
    let palette = palette_colors(options.palette.as_deref())?;
    let plot = BarPlot::new()
        .with_bars(
            result
                .groups
                .iter()
                .map(|group| (group.group_name.clone(), group.event_rate))
                .collect::<Vec<_>>(),
        )
        .with_color(palette[0].clone());
    render_chart(
        vec![Plot::Bar(plot)],
        options,
        &format!("{} survival event rate", result.gene),
        "Group",
        "Event rate",
    )
}

fn render_chart(
    plots: Vec<Plot>,
    options: &ChartRenderOptions,
    default_title: &str,
    x_label: &str,
    y_label: &str,
) -> Result<String, BioMcpError> {
    let target = output_target(options)?;
    let palette = palette_from_name(options.palette.as_deref())?;
    let theme = theme_from_name(
        options.theme.as_deref(),
        matches!(target, OutputTarget::Terminal),
    )?;

    let mut layout = Layout::auto_from_plots(&plots)
        .with_x_label(x_label)
        .with_y_label(y_label)
        .with_theme(theme)
        .with_palette(palette);
    let title = options.title.as_deref().unwrap_or(default_title);
    if !title.trim().is_empty() {
        layout = layout.with_title(title);
    }

    let scene = render_multiple(plots, layout);
    match target {
        OutputTarget::Terminal => {
            Ok(TerminalBackend::new(TERMINAL_COLS, TERMINAL_ROWS).render_scene(&scene))
        }
        OutputTarget::Svg(path) => {
            let svg = SvgBackend.render_scene(&scene);
            fs::write(&path, svg)?;
            Ok(format!("Wrote SVG chart to {}", path.display()))
        }
        OutputTarget::Png(path) => write_png(&scene, &path),
        OutputTarget::InlineSvg => Ok(SvgBackend.render_scene(&scene)),
    }
}

fn output_target(options: &ChartRenderOptions) -> Result<OutputTarget, BioMcpError> {
    if options.inline_svg {
        if options.output.is_some() {
            return Err(BioMcpError::InvalidArgument(
                "MCP inline chart output cannot be combined with file output.".into(),
            ));
        }
        return Ok(OutputTarget::InlineSvg);
    }
    if let Some(path) = options.output.clone() {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase());
        return match extension.as_deref() {
            Some("svg") => Ok(OutputTarget::Svg(path)),
            Some("png") => Ok(OutputTarget::Png(path)),
            Some(other) => Err(BioMcpError::InvalidArgument(format!(
                "Unsupported output format '.{other}'. Use .svg or .png"
            ))),
            None => Err(BioMcpError::InvalidArgument(
                "Unsupported output format ''. Use .svg or .png".into(),
            )),
        };
    }
    Ok(OutputTarget::Terminal)
}

fn write_png(scene: &kuva::render::render::Scene, path: &PathBuf) -> Result<String, BioMcpError> {
    #[cfg(feature = "charts-png")]
    {
        let bytes = PngBackend::new()
            .render_scene(scene)
            .map_err(BioMcpError::InvalidArgument)?;
        fs::write(path, bytes)?;
        Ok(format!("Wrote PNG chart to {}", path.display()))
    }
    #[cfg(not(feature = "charts-png"))]
    {
        let _ = scene;
        let _ = path;
        Err(BioMcpError::InvalidArgument(
            "PNG output requires BioMCP to be built with --features charts-png".into(),
        ))
    }
}

fn theme_from_name(name: Option<&str>, terminal_default: bool) -> Result<Theme, BioMcpError> {
    match name.map(|value| value.trim().to_ascii_lowercase()) {
        None => Ok(if terminal_default {
            Theme::dark()
        } else {
            Theme::light()
        }),
        Some(value) if value == "light" => Ok(Theme::light()),
        Some(value) if value == "dark" => Ok(Theme::dark()),
        Some(value) if value == "solarized" => Ok(Theme::solarized()),
        Some(value) if value == "minimal" => Ok(Theme::minimal()),
        Some(other) => Err(BioMcpError::InvalidArgument(format!(
            "Unknown chart theme '{other}'. Valid themes: light, dark, solarized, minimal"
        ))),
    }
}

fn palette_from_name(name: Option<&str>) -> Result<Palette, BioMcpError> {
    match name.map(|value| value.trim().to_ascii_lowercase()) {
        None => Ok(Palette::category10()),
        Some(value) if value == "wong" => Ok(Palette::wong()),
        Some(value) if value == "okabe-ito" || value == "okabe_ito" => Ok(Palette::okabe_ito()),
        Some(value) if value == "tol-bright" || value == "tol_bright" => Ok(Palette::tol_bright()),
        Some(value) if value == "tol-muted" || value == "tol_muted" => Ok(Palette::tol_muted()),
        Some(value) if value == "tol-light" || value == "tol_light" => Ok(Palette::tol_light()),
        Some(value) if value == "ibm" => Ok(Palette::ibm()),
        Some(value) if value == "deuteranopia" => Ok(Palette::deuteranopia()),
        Some(value) if value == "protanopia" => Ok(Palette::protanopia()),
        Some(value) if value == "tritanopia" => Ok(Palette::tritanopia()),
        Some(value) if value == "category10" => Ok(Palette::category10()),
        Some(value) if value == "pastel" => Ok(Palette::pastel()),
        Some(value) if value == "bold" => Ok(Palette::bold()),
        Some(other) => Err(BioMcpError::InvalidArgument(format!(
            "Unknown chart palette '{other}'. Valid palettes: wong, okabe-ito, tol-bright, tol-muted, tol-light, ibm, deuteranopia, protanopia, tritanopia, category10, pastel, bold"
        ))),
    }
}

fn palette_colors(name: Option<&str>) -> Result<Vec<String>, BioMcpError> {
    Ok(palette_from_name(name)?.colors().to_vec())
}

fn ensure_non_empty(values: &[f64], label: &str) -> Result<(), BioMcpError> {
    if values.is_empty() {
        return Err(BioMcpError::InvalidArgument(format!(
            "{label} chart requires at least one numeric value."
        )));
    }
    Ok(())
}

fn numeric_range(values: &[f64]) -> Result<(f64, f64), BioMcpError> {
    ensure_non_empty(values, "Histogram")?;
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if min == max {
        Ok((min - 0.5, max + 0.5))
    } else {
        Ok((min, max))
    }
}

fn suggest_bins(sample_count: usize) -> usize {
    ((sample_count as f64).sqrt().round() as usize).clamp(5, 20)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::entities::study::{
        CnaDistributionResult, CoOccurrencePair, CoOccurrenceResult, MutationComparisonResult,
        MutationFrequencyResult, MutationGroupStats, SampleUniverseBasis, StudyQueryType,
        SurvivalEndpoint, SurvivalGroupResult, SurvivalResult,
    };

    use super::{
        ChartRenderOptions, render_cna_chart, render_co_occurrence_chart,
        render_expression_compare_chart, render_expression_density_chart,
        render_expression_histogram_chart, render_mutation_compare_chart,
        render_mutation_frequency_chart, render_survival_chart, validate_compare_chart_type,
        validate_query_chart_type, validate_standalone_chart_type,
    };
    use crate::cli::ChartType;

    fn terminal_options() -> ChartRenderOptions {
        ChartRenderOptions {
            terminal: true,
            inline_svg: false,
            output: None,
            title: None,
            theme: None,
            palette: None,
        }
    }

    struct TestOutputDir {
        path: PathBuf,
    }

    impl TestOutputDir {
        fn new() -> Self {
            let suffix = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "biomcp-chart-tests-{}-{suffix}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("temp chart dir");
            Self { path }
        }

        fn svg_path(&self, name: &str) -> PathBuf {
            self.path.join(name)
        }
    }

    impl Drop for TestOutputDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn svg_options(path: PathBuf) -> ChartRenderOptions {
        ChartRenderOptions {
            terminal: false,
            inline_svg: false,
            output: Some(path),
            title: Some("Example".into()),
            theme: Some("minimal".into()),
            palette: Some("wong".into()),
        }
    }

    fn inline_svg_options() -> ChartRenderOptions {
        ChartRenderOptions {
            terminal: false,
            inline_svg: true,
            output: None,
            title: Some("Example".into()),
            theme: Some("minimal".into()),
            palette: Some("wong".into()),
        }
    }

    #[test]
    fn query_chart_validation_lists_valid_types() {
        let err = validate_query_chart_type(StudyQueryType::Mutations, ChartType::Violin)
            .expect_err("violin should be rejected for mutation queries");
        let msg = err.to_string();
        assert!(msg.contains("study query --type mutations"));
        assert!(msg.contains("bar"));
        assert!(msg.contains("pie"));
    }

    #[test]
    fn compare_chart_validation_lists_valid_types() {
        let err = validate_compare_chart_type("expression", ChartType::Pie)
            .expect_err("pie should be rejected for expression compare");
        let msg = err.to_string();
        assert!(msg.contains("study compare --type expression"));
        assert!(msg.contains("box"));
        assert!(msg.contains("violin"));
        assert!(msg.contains("ridgeline"));
    }

    #[test]
    fn standalone_chart_validation_rejects_invalid_survival_chart() {
        let err = validate_standalone_chart_type(
            "study survival",
            ChartType::Histogram,
            &[ChartType::Bar],
        )
        .expect_err("histogram should be rejected for survival");
        let msg = err.to_string();
        assert!(msg.contains("study survival"));
        assert!(msg.contains("bar"));
    }

    #[test]
    fn inline_svg_target_returns_svg_markup() {
        let mutation = MutationFrequencyResult {
            study_id: "demo".into(),
            gene: "TP53".into(),
            mutation_count: 7,
            unique_samples: 5,
            total_samples: 20,
            frequency: 0.25,
            top_variant_classes: vec![
                ("Missense_Mutation".into(), 4),
                ("Nonsense_Mutation".into(), 3),
            ],
            top_protein_changes: vec![("R175H".into(), 3)],
        };

        let svg = render_mutation_frequency_chart(&mutation, ChartType::Bar, &inline_svg_options())
            .expect("inline svg should render");

        assert!(svg.contains("<svg"));
        assert!(svg.contains("Example"));
    }

    #[test]
    fn bar_family_renderers_produce_svg() {
        let output_dir = TestOutputDir::new();
        let mutation = MutationFrequencyResult {
            study_id: "demo".into(),
            gene: "TP53".into(),
            mutation_count: 7,
            unique_samples: 5,
            total_samples: 20,
            frequency: 0.25,
            top_variant_classes: vec![
                ("Missense_Mutation".into(), 4),
                ("Nonsense_Mutation".into(), 3),
            ],
            top_protein_changes: vec![("R175H".into(), 3)],
        };
        let cna = CnaDistributionResult {
            study_id: "demo".into(),
            gene: "ERBB2".into(),
            total_samples: 20,
            deep_deletion: 1,
            shallow_deletion: 2,
            diploid: 10,
            gain: 3,
            amplification: 4,
        };
        let mutation_compare = MutationComparisonResult {
            study_id: "demo".into(),
            stratify_gene: "TP53".into(),
            target_gene: "PIK3CA".into(),
            groups: vec![
                MutationGroupStats {
                    group_name: "TP53-mutant".into(),
                    sample_count: 8,
                    mutated_count: 4,
                    mutation_rate: 0.5,
                },
                MutationGroupStats {
                    group_name: "TP53-wildtype".into(),
                    sample_count: 12,
                    mutated_count: 3,
                    mutation_rate: 0.25,
                },
            ],
        };
        let survival = SurvivalResult {
            study_id: "demo".into(),
            gene: "TP53".into(),
            endpoint: SurvivalEndpoint::Os,
            groups: vec![
                SurvivalGroupResult {
                    group_name: "TP53-mutant".into(),
                    n_patients: 8,
                    n_events: 3,
                    n_censored: 5,
                    km_median_months: Some(18.0),
                    survival_1yr: Some(0.75),
                    survival_3yr: Some(0.5),
                    survival_5yr: Some(0.25),
                    event_rate: 0.375,
                },
                SurvivalGroupResult {
                    group_name: "TP53-wildtype".into(),
                    n_patients: 12,
                    n_events: 2,
                    n_censored: 10,
                    km_median_months: None,
                    survival_1yr: Some(0.9),
                    survival_3yr: Some(0.8),
                    survival_5yr: Some(0.7),
                    event_rate: 0.1667,
                },
            ],
            log_rank_p: Some(0.02),
        };
        let mutation_path = output_dir.svg_path("mutation.svg");
        let cna_path = output_dir.svg_path("cna.svg");
        let compare_path = output_dir.svg_path("compare.svg");
        let survival_path = output_dir.svg_path("survival.svg");

        assert!(
            render_mutation_frequency_chart(
                &mutation,
                ChartType::Bar,
                &svg_options(mutation_path.clone())
            )
            .expect("mutation svg")
            .contains(mutation_path.to_string_lossy().as_ref())
        );
        assert!(
            render_cna_chart(&cna, ChartType::Bar, &svg_options(cna_path.clone()))
                .expect("cna svg")
                .contains(cna_path.to_string_lossy().as_ref())
        );
        assert!(
            render_mutation_compare_chart(
                &mutation_compare,
                ChartType::Bar,
                &svg_options(compare_path.clone()),
            )
            .expect("compare svg")
            .contains(compare_path.to_string_lossy().as_ref())
        );
        assert!(
            render_survival_chart(
                &survival,
                ChartType::Bar,
                &svg_options(survival_path.clone())
            )
            .expect("survival svg")
            .contains(survival_path.to_string_lossy().as_ref())
        );
        for path in [mutation_path, cna_path, compare_path, survival_path] {
            assert!(path.exists(), "expected {} to exist", path.display());
        }
    }

    #[test]
    fn pie_histogram_density_and_distribution_renderers_produce_terminal_output() {
        let co_occurrence = CoOccurrenceResult {
            study_id: "demo".into(),
            genes: vec!["TP53".into(), "KRAS".into()],
            total_samples: 20,
            sample_universe_basis: SampleUniverseBasis::ClinicalSampleFile,
            pairs: vec![CoOccurrencePair {
                gene_a: "TP53".into(),
                gene_b: "KRAS".into(),
                both_mutated: 3,
                a_only: 5,
                b_only: 2,
                neither: 10,
                log_odds_ratio: Some(0.7),
                p_value: Some(0.04),
            }],
        };
        let expression_compare = vec![
            ("TP53-mutant".to_string(), vec![1.0, 1.5, 2.0, 2.5]),
            ("TP53-wildtype".to_string(), vec![0.2, 0.5, 0.8, 1.2]),
        ];

        let pie = render_co_occurrence_chart(&co_occurrence, ChartType::Pie, &terminal_options())
            .expect("co-occurrence pie");
        let hist = render_expression_histogram_chart(
            "demo",
            "ERBB2",
            &[0.1, 0.3, 0.9, 1.2, 1.8],
            &terminal_options(),
        )
        .expect("histogram");
        let density = render_expression_density_chart(
            "demo",
            "ERBB2",
            &[0.1, 0.3, 0.9, 1.2, 1.8],
            &terminal_options(),
        )
        .expect("density");
        let violin = render_expression_compare_chart(
            "demo",
            "TP53",
            "ERBB2",
            &expression_compare,
            ChartType::Violin,
            &terminal_options(),
        )
        .expect("violin");

        assert!(!pie.trim().is_empty());
        assert!(!hist.trim().is_empty());
        assert!(!density.trim().is_empty());
        assert!(!violin.trim().is_empty());
    }
}
