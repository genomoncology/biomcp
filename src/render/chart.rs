use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use kuva::backend::svg::SvgBackend;
use kuva::backend::terminal::TerminalBackend;
use kuva::plot::{
    BarPlot, BoxPlot, ColorMap, DensityPlot, Heatmap, Histogram, LinePlot, PiePlot, RidgelinePlot,
    ScatterPlot, ViolinPlot,
};
use kuva::prelude::{Layout, Palette, PieLabelPosition, Plot, Theme, TickFormat, render_multiple};

#[cfg(feature = "charts-png")]
use kuva::PngBackend;

use crate::cli::{ChartArgs, ChartType};
use crate::entities::study::{
    CnaDistributionResult, CoOccurrenceResult, MutationComparisonResult, MutationFrequencyResult,
    StudyQueryType, SurvivalResult,
};
use crate::error::BioMcpError;

const DEFAULT_TERMINAL_COLS: usize = 100;
const DEFAULT_TERMINAL_ROWS: usize = 32;
const DEFAULT_PNG_SCALE: f32 = 2.0;

pub(crate) const TERMINAL_SIZE_FLAGS_ERROR: &str = "--cols/--rows require terminal chart output";
pub(crate) const IMAGE_DIMENSION_FLAGS_ERROR: &str =
    "--width/--height require SVG, PNG, or MCP inline SVG chart output";
pub(crate) const PNG_SCALE_FLAGS_ERROR: &str = "--scale requires PNG chart output";

fn chart_text(value: &str) -> String {
    crate::render::human::sanitize_inline(value)
}

fn display_mutation_class(label: &str) -> Cow<'_, str> {
    match label.trim() {
        "Missense_Mutation" => Cow::Borrowed("Missense"),
        "Nonsense_Mutation" => Cow::Borrowed("Nonsense"),
        "Frame_Shift_Del" => Cow::Borrowed("Frameshift Del"),
        "Frame_Shift_Ins" => Cow::Borrowed("Frameshift Ins"),
        "Splice_Site" => Cow::Borrowed("Splice"),
        "In_Frame_Del" => Cow::Borrowed("In-Frame Del"),
        "In_Frame_Ins" => Cow::Borrowed("In-Frame Ins"),
        "Nonstop_Mutation" => Cow::Borrowed("Nonstop"),
        "Translation_Start_Site" => Cow::Borrowed("Start Site"),
        "Amp" => Cow::Borrowed("Amp"),
        "Amplification" => Cow::Borrowed("Amp"),
        other => Cow::Borrowed(other),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ChartRenderOptions {
    pub terminal: bool,
    pub inline_svg: bool,
    pub output: Option<PathBuf>,
    pub title: Option<String>,
    pub theme: Option<String>,
    pub palette: Option<String>,
    pub cols: Option<usize>,
    pub rows: Option<usize>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub scale: Option<f32>,
}

impl From<&ChartArgs> for ChartRenderOptions {
    fn from(args: &ChartArgs) -> Self {
        Self {
            terminal: args.terminal,
            inline_svg: args.mcp_inline,
            output: args.output.clone(),
            title: args.title.clone(),
            theme: args.theme.clone(),
            palette: args.palette.clone(),
            cols: args.cols,
            rows: args.rows,
            width: args.width,
            height: args.height,
            scale: args.scale,
        }
    }
}

enum OutputTarget {
    Terminal { cols: usize, rows: usize },
    Svg(PathBuf),
    Png { path: PathBuf, scale: f32 },
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
            &[ChartType::Bar, ChartType::Pie, ChartType::Waterfall],
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
        StudyQueryType::StructuralVariants => {
            validate_standalone_chart_type("study query --type sv", chart_type, &[])
        }
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
            &[
                ChartType::Box,
                ChartType::Violin,
                ChartType::Ridgeline,
                ChartType::Scatter,
            ],
        ),
        "mutations" | "mutation" => validate_standalone_chart_type(
            "study compare --type mutations",
            chart_type,
            &[ChartType::Bar, ChartType::StackedBar],
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
    if valid_types.is_empty() {
        return Err(BioMcpError::InvalidArgument(format!(
            "charts are not supported for '{command_label}'"
        )));
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
                    .map(|(label, count)| {
                        (chart_text(&display_mutation_class(label)), *count as f64)
                    })
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
                    .map(|(label, count)| {
                        (chart_text(&display_mutation_class(label)), *count as f64)
                    })
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

pub(crate) fn render_mutation_waterfall_chart(
    study_id: &str,
    gene: &str,
    sample_counts: &[(String, usize)],
    options: &ChartRenderOptions,
) -> Result<String, BioMcpError> {
    if sample_counts.is_empty() {
        return Err(BioMcpError::InvalidArgument(format!(
            "{gene} mutation waterfall chart requires at least one mutated sample."
        )));
    }
    let palette = palette_colors(options.palette.as_deref())?;
    let plot = BarPlot::new()
        .with_bars(
            sample_counts
                .iter()
                .map(|(sample_id, count)| (chart_text(sample_id), *count as f64))
                .collect::<Vec<_>>(),
        )
        .with_color(palette[0].clone());
    render_chart(
        vec![Plot::Bar(plot)],
        options,
        &format!("{gene} mutation burden by sample ({study_id})"),
        "Sample",
        "Mutation count",
    )
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
        ("Deep Del".to_string(), result.deep_deletion as f64),
        ("Shallow Del".to_string(), result.shallow_deletion as f64),
        ("Diploid".to_string(), result.diploid as f64),
        ("Gain".to_string(), result.gain as f64),
        ("Amplification".to_string(), result.amplification as f64),
    ];
    let title = format!("{} CNA distribution", result.gene);
    match chart_type {
        ChartType::Bar => {
            let plot = BarPlot::new()
                .with_bars(categories)
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
                plot = plot.with_group(chart_text(label), values.iter().copied());
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
                plot = plot.with_group(chart_text(label), values.iter().copied());
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
                    chart_text(label),
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

pub(crate) fn render_expression_scatter_chart(
    study_id: &str,
    gene_x: &str,
    gene_y: &str,
    points: &[(f64, f64)],
    options: &ChartRenderOptions,
) -> Result<String, BioMcpError> {
    if points.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "scatter chart requires at least one paired sample.".into(),
        ));
    }
    let palette = palette_colors(options.palette.as_deref())?;
    let plot = ScatterPlot::new()
        .with_data(points.iter().copied())
        .with_color(palette[0].clone());
    render_chart(
        vec![Plot::Scatter(plot)],
        options,
        &format!("{gene_x} vs {gene_y} expression ({study_id})"),
        gene_x,
        gene_y,
    )
}

pub(crate) fn render_mutation_compare_chart(
    result: &MutationComparisonResult,
    chart_type: ChartType,
    options: &ChartRenderOptions,
) -> Result<String, BioMcpError> {
    validate_standalone_chart_type(
        "study compare --type mutations",
        chart_type,
        &[ChartType::Bar, ChartType::StackedBar],
    )?;
    let palette = palette_colors(options.palette.as_deref())?;
    match chart_type {
        ChartType::Bar => {
            let plot = BarPlot::new()
                .with_bars(
                    result
                        .groups
                        .iter()
                        .map(|group| (chart_text(&group.group_name), group.mutation_rate))
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
        ChartType::StackedBar => {
            let mut plot = BarPlot::new();
            for group in &result.groups {
                let not_mutated = group.sample_count.saturating_sub(group.mutated_count);
                plot = plot.with_group(
                    chart_text(&group.group_name),
                    vec![
                        (group.mutated_count as f64, palette[0].as_str()),
                        (not_mutated as f64, palette[1].as_str()),
                    ],
                );
            }
            plot = plot
                .with_stacked()
                .with_legend(vec!["Mutated", "Not mutated"]);
            render_chart_with_layout(
                vec![Plot::Bar(plot)],
                options,
                &format!(
                    "{} mutation status by {}",
                    result.target_gene, result.stratify_gene
                ),
                "Group",
                "Samples",
                |layout| {
                    layout
                        .with_y_tick_step(1.0)
                        .with_y_tick_format(TickFormat::Integer)
                },
            )
        }
        other => Err(BioMcpError::InvalidArgument(format!(
            "Unsupported mutation comparison chart type '{other}'"
        ))),
    }
}

pub(crate) fn render_co_occurrence_chart(
    result: &CoOccurrenceResult,
    chart_type: ChartType,
    options: &ChartRenderOptions,
) -> Result<String, BioMcpError> {
    validate_standalone_chart_type(
        "study co-occurrence",
        chart_type,
        &[ChartType::Bar, ChartType::Pie, ChartType::Heatmap],
    )?;
    if result.pairs.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Co-occurrence chart requires at least one gene pair.".into(),
        ));
    }
    match chart_type {
        ChartType::Bar => {
            let palette = palette_colors(options.palette.as_deref())?;
            let plot = BarPlot::new()
                .with_bars(
                    result
                        .pairs
                        .iter()
                        .map(|pair| {
                            (
                                chart_text(&format!("{}/{}", pair.gene_a, pair.gene_b)),
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
            let palette = palette_colors(options.palette.as_deref())?;
            let pair = &result.pairs[0];
            let mut plot = PiePlot::new()
                .with_legend("Contingency")
                .with_percent()
                .with_label_position(PieLabelPosition::Auto);
            for (idx, (label, value)) in [
                ("Both mutated".to_string(), pair.both_mutated as f64),
                (
                    chart_text(&format!("{} only", pair.gene_a)),
                    pair.a_only as f64,
                ),
                (
                    chart_text(&format!("{} only", pair.gene_b)),
                    pair.b_only as f64,
                ),
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
        ChartType::Heatmap => render_co_occurrence_heatmap(result, options),
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
    validate_standalone_chart_type(
        "study survival",
        chart_type,
        &[ChartType::Bar, ChartType::Survival],
    )?;
    let palette = palette_colors(options.palette.as_deref())?;
    match chart_type {
        ChartType::Bar => {
            let plot = BarPlot::new()
                .with_bars(
                    result
                        .groups
                        .iter()
                        .map(|group| (chart_text(&group.group_name), group.event_rate))
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
        ChartType::Survival => {
            let plots = result
                .groups
                .iter()
                .enumerate()
                .filter(|(_, group)| !group.km_curve_points.is_empty())
                .map(|(idx, group)| {
                    Plot::Line(
                        LinePlot::new()
                            .with_data(group.km_curve_points.iter().copied())
                            .with_step()
                            .with_color(palette[idx % palette.len()].clone())
                            .with_legend(chart_text(&group.group_name)),
                    )
                })
                .collect::<Vec<_>>();
            if plots.is_empty() {
                return Err(BioMcpError::InvalidArgument(
                    "study survival --chart survival requires at least one non-empty KM curve."
                        .into(),
                ));
            }
            render_chart(
                plots,
                options,
                &format!("{} {} Kaplan-Meier", result.gene, result.endpoint.label()),
                "Time (months)",
                "Survival probability",
            )
        }
        other => Err(BioMcpError::InvalidArgument(format!(
            "Unsupported survival chart type '{other}'"
        ))),
    }
}

fn render_co_occurrence_heatmap(
    result: &CoOccurrenceResult,
    options: &ChartRenderOptions,
) -> Result<String, BioMcpError> {
    reject_palette_override("study co-occurrence --chart heatmap", options)?;

    let gene_count = result.genes.len();
    let mut matrix = vec![vec![0.0; gene_count]; gene_count];
    let gene_index = result
        .genes
        .iter()
        .enumerate()
        .map(|(idx, gene)| (gene.as_str(), idx))
        .collect::<HashMap<_, _>>();

    for pair in &result.pairs {
        let i = *gene_index.get(pair.gene_a.as_str()).ok_or_else(|| {
            BioMcpError::InvalidArgument(format!(
                "Co-occurrence pair references unknown gene '{}'.",
                pair.gene_a
            ))
        })?;
        let j = *gene_index.get(pair.gene_b.as_str()).ok_or_else(|| {
            BioMcpError::InvalidArgument(format!(
                "Co-occurrence pair references unknown gene '{}'.",
                pair.gene_b
            ))
        })?;
        matrix[i][j] = pair.both_mutated as f64;
        matrix[j][i] = pair.both_mutated as f64;
    }

    let genes = result
        .genes
        .iter()
        .map(|gene| chart_text(gene))
        .collect::<Vec<_>>();
    let plot = Heatmap::new()
        .with_data(matrix)
        .with_labels(genes.clone(), genes.clone())
        .with_color_map(ColorMap::Viridis)
        .with_legend("Both-mutated sample count");
    let plots = vec![Plot::Heatmap(plot)];
    let target = output_target(options)?;
    let terminal_default = !options.inline_svg && options.output.is_none();
    let theme = theme_from_name(options.theme.as_deref(), terminal_default)?;

    let mut layout = Layout::auto_from_plots(&plots)
        .with_x_categories(genes.clone())
        .with_y_categories(genes)
        .with_x_label("Gene")
        .with_y_label("Gene")
        .with_theme(theme);
    let default_title = format!("Co-mutation heatmap ({})", result.study_id);
    let title = chart_text(options.title.as_deref().unwrap_or(default_title.as_str()));
    if !title.trim().is_empty() {
        layout = layout.with_title(title);
    }
    let layout = apply_layout_dimensions(layout, options);

    render_layout(plots, layout, target)
}

fn render_chart(
    plots: Vec<Plot>,
    options: &ChartRenderOptions,
    default_title: &str,
    x_label: &str,
    y_label: &str,
) -> Result<String, BioMcpError> {
    render_chart_with_layout(
        plots,
        options,
        default_title,
        x_label,
        y_label,
        |layout: Layout| layout,
    )
}

fn render_chart_with_layout<F>(
    plots: Vec<Plot>,
    options: &ChartRenderOptions,
    default_title: &str,
    x_label: &str,
    y_label: &str,
    configure_layout: F,
) -> Result<String, BioMcpError>
where
    F: FnOnce(Layout) -> Layout,
{
    let target = output_target(options)?;
    let palette = palette_from_name(options.palette.as_deref())?;
    let terminal_default = !options.inline_svg && options.output.is_none();
    let theme = theme_from_name(options.theme.as_deref(), terminal_default)?;

    let mut layout = Layout::auto_from_plots(&plots)
        .with_x_label(chart_text(x_label))
        .with_y_label(chart_text(y_label))
        .with_theme(theme)
        .with_palette(palette);
    let title = chart_text(options.title.as_deref().unwrap_or(default_title));
    if !title.trim().is_empty() {
        layout = layout.with_title(title);
    }

    let layout = apply_layout_dimensions(configure_layout(layout), options);
    render_layout(plots, layout, target)
}

fn apply_layout_dimensions(mut layout: Layout, options: &ChartRenderOptions) -> Layout {
    if let Some(width) = options.width {
        layout = layout.with_width(f64::from(width));
    }
    if let Some(height) = options.height {
        layout = layout.with_height(f64::from(height));
    }
    layout
}

fn render_layout(
    plots: Vec<Plot>,
    layout: Layout,
    target: OutputTarget,
) -> Result<String, BioMcpError> {
    let scene = render_multiple(plots, layout);
    match target {
        OutputTarget::Terminal { cols, rows } => {
            Ok(TerminalBackend::new(cols, rows).render_scene(&scene))
        }
        OutputTarget::Svg(path) => {
            let svg = SvgBackend.render_scene(&scene);
            fs::write(&path, svg)?;
            Ok(format!("Wrote SVG chart to {}", path.display()))
        }
        OutputTarget::Png { path, scale } => write_png(&scene, &path, scale),
        OutputTarget::InlineSvg => Ok(SvgBackend.render_scene(&scene)),
    }
}

fn reject_palette_override(
    command_label: &str,
    options: &ChartRenderOptions,
) -> Result<(), BioMcpError> {
    if options.palette.is_some() {
        return Err(BioMcpError::InvalidArgument(format!(
            "--palette is not supported for '{command_label}'. Omit --palette; heatmaps use a fixed continuous colormap."
        )));
    }
    Ok(())
}

fn output_target(options: &ChartRenderOptions) -> Result<OutputTarget, BioMcpError> {
    if options.inline_svg {
        if options.output.is_some() {
            return Err(BioMcpError::InvalidArgument(
                "MCP inline chart output cannot be combined with file output.".into(),
            ));
        }
        if options.cols.is_some() || options.rows.is_some() {
            return Err(BioMcpError::InvalidArgument(
                TERMINAL_SIZE_FLAGS_ERROR.into(),
            ));
        }
        if options.scale.is_some() {
            return Err(BioMcpError::InvalidArgument(PNG_SCALE_FLAGS_ERROR.into()));
        }
        return Ok(OutputTarget::InlineSvg);
    }
    if let Some(path) = options.output.clone() {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase());
        return match extension.as_deref() {
            Some("svg") => {
                if options.cols.is_some() || options.rows.is_some() {
                    return Err(BioMcpError::InvalidArgument(
                        TERMINAL_SIZE_FLAGS_ERROR.into(),
                    ));
                }
                if options.scale.is_some() {
                    return Err(BioMcpError::InvalidArgument(PNG_SCALE_FLAGS_ERROR.into()));
                }
                Ok(OutputTarget::Svg(path))
            }
            Some("png") => {
                if options.cols.is_some() || options.rows.is_some() {
                    return Err(BioMcpError::InvalidArgument(
                        TERMINAL_SIZE_FLAGS_ERROR.into(),
                    ));
                }
                Ok(OutputTarget::Png {
                    path,
                    scale: options.scale.unwrap_or(DEFAULT_PNG_SCALE),
                })
            }
            Some(other) => Err(BioMcpError::InvalidArgument(format!(
                "Unsupported output format '.{other}'. Use .svg or .png"
            ))),
            None => Err(BioMcpError::InvalidArgument(
                "Unsupported output format ''. Use .svg or .png".into(),
            )),
        };
    }
    if options.width.is_some() || options.height.is_some() {
        return Err(BioMcpError::InvalidArgument(
            IMAGE_DIMENSION_FLAGS_ERROR.into(),
        ));
    }
    if options.scale.is_some() {
        return Err(BioMcpError::InvalidArgument(PNG_SCALE_FLAGS_ERROR.into()));
    }
    Ok(OutputTarget::Terminal {
        cols: options.cols.unwrap_or(DEFAULT_TERMINAL_COLS),
        rows: options.rows.unwrap_or(DEFAULT_TERMINAL_ROWS),
    })
}

fn write_png(
    scene: &kuva::render::render::Scene,
    path: &PathBuf,
    scale: f32,
) -> Result<String, BioMcpError> {
    #[cfg(feature = "charts-png")]
    {
        let bytes = PngBackend::new()
            .with_scale(scale)
            .render_scene(scene)
            .map_err(BioMcpError::InvalidArgument)?;
        fs::write(path, bytes)?;
        Ok(format!("Wrote PNG chart to {}", path.display()))
    }
    #[cfg(not(feature = "charts-png"))]
    {
        let _ = scene;
        let _ = path;
        let _ = scale;
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

    use crate::entities::study::{
        CnaDistributionResult, CoOccurrencePair, CoOccurrenceResult, MutationComparisonResult,
        MutationFrequencyResult, MutationGroupStats, SampleUniverseBasis, StudyQueryType,
        SurvivalEndpoint, SurvivalGroupResult, SurvivalResult,
    };
    use crate::test_support::TempDirGuard;

    use super::{
        ChartRenderOptions, chart_text, display_mutation_class, render_cna_chart,
        render_co_occurrence_chart, render_expression_compare_chart,
        render_expression_density_chart, render_expression_histogram_chart,
        render_expression_scatter_chart, render_mutation_compare_chart,
        render_mutation_frequency_chart, render_mutation_waterfall_chart, render_survival_chart,
        validate_compare_chart_type, validate_query_chart_type, validate_standalone_chart_type,
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
            cols: None,
            rows: None,
            width: None,
            height: None,
            scale: None,
        }
    }

    fn custom_terminal_options(cols: usize, rows: usize) -> ChartRenderOptions {
        ChartRenderOptions {
            cols: Some(cols),
            rows: Some(rows),
            ..terminal_options()
        }
    }

    struct TestOutputDir {
        _guard: TempDirGuard,
        path: PathBuf,
    }

    impl TestOutputDir {
        fn new() -> Self {
            let guard = TempDirGuard::new("chart-tests");
            let path = guard.path().to_path_buf();
            Self {
                _guard: guard,
                path,
            }
        }

        fn svg_path(&self, name: &str) -> PathBuf {
            self.path.join(name)
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
            cols: None,
            rows: None,
            width: None,
            height: None,
            scale: None,
        }
    }

    fn svg_heatmap_options(path: PathBuf) -> ChartRenderOptions {
        ChartRenderOptions {
            terminal: false,
            inline_svg: false,
            output: Some(path),
            title: None,
            theme: Some("minimal".into()),
            palette: None,
            cols: None,
            rows: None,
            width: None,
            height: None,
            scale: None,
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
            cols: None,
            rows: None,
            width: None,
            height: None,
            scale: None,
        }
    }

    fn inline_svg_heatmap_options() -> ChartRenderOptions {
        ChartRenderOptions {
            terminal: false,
            inline_svg: true,
            output: None,
            title: None,
            theme: Some("minimal".into()),
            palette: None,
            cols: None,
            rows: None,
            width: None,
            height: None,
            scale: None,
        }
    }

    fn inline_svg_auto_title_options() -> ChartRenderOptions {
        ChartRenderOptions {
            terminal: false,
            inline_svg: true,
            output: None,
            title: None,
            theme: Some("minimal".into()),
            palette: Some("wong".into()),
            cols: None,
            rows: None,
            width: None,
            height: None,
            scale: None,
        }
    }

    fn sized_inline_svg_options(width: u32, height: u32) -> ChartRenderOptions {
        ChartRenderOptions {
            width: Some(width),
            height: Some(height),
            ..inline_svg_options()
        }
    }

    fn strip_ansi(input: &str) -> String {
        let mut output = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\u{1b}' {
                if matches!(chars.peek(), Some('[')) {
                    chars.next();
                }
                for next in chars.by_ref() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
                continue;
            }
            output.push(ch);
        }
        output
    }

    #[test]
    fn chart_text_removes_untrusted_controls_without_touching_unicode() {
        assert_eq!(
            chart_text("Control\u{7}Title\u{1b}[31m α"),
            "Control Title α"
        );
    }

    #[test]
    fn display_mutation_class_maps_known_and_passes_through_unknown() {
        assert_eq!(display_mutation_class("Missense_Mutation"), "Missense");
        assert_eq!(display_mutation_class("Frame_Shift_Del"), "Frameshift Del");
        assert_eq!(display_mutation_class("Splice_Site"), "Splice");
        assert_eq!(display_mutation_class("Amplification"), "Amp");
        // Unknown labels pass through unchanged
        assert_eq!(display_mutation_class("CUSTOM_LABEL"), "CUSTOM_LABEL");
        assert_eq!(display_mutation_class("Some_Other"), "Some_Other");
    }

    #[test]
    fn render_survival_chart_returns_error_when_all_groups_have_empty_km_points() {
        let survival = SurvivalResult {
            study_id: "demo".into(),
            gene: "TP53".into(),
            endpoint: SurvivalEndpoint::Os,
            groups: vec![SurvivalGroupResult {
                group_name: "TP53-mutant".into(),
                n_patients: 5,
                n_events: 2,
                n_censored: 3,
                km_median_months: None,
                survival_1yr: None,
                survival_3yr: None,
                survival_5yr: None,
                event_rate: 0.4,
                km_curve_points: vec![],
            }],
            log_rank_p: None,
        };
        let err = render_survival_chart(&survival, ChartType::Survival, &inline_svg_options())
            .expect_err("should fail when all groups have empty km_curve_points");
        assert!(
            err.to_string().contains("non-empty KM curve"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn query_chart_validation_lists_valid_types() {
        let err = validate_query_chart_type(StudyQueryType::Mutations, ChartType::Violin)
            .expect_err("violin should be rejected for mutation queries");
        let msg = err.to_string();
        assert!(msg.contains("study query --type mutations"));
        assert!(msg.contains("bar"));
        assert!(msg.contains("pie"));
        assert!(msg.contains("waterfall"));
    }

    #[test]
    fn structural_variant_chart_validation_reports_unsupported_surface() {
        let err = validate_query_chart_type(StudyQueryType::StructuralVariants, ChartType::Bar)
            .expect_err("charts should be rejected for structural variant queries");
        let msg = err.to_string();
        assert!(msg.contains("charts are not supported"));
        assert!(msg.contains("study query --type sv"));
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
        assert!(msg.contains("scatter"));
    }

    #[test]
    fn mutation_compare_validation_lists_stacked_bar() {
        let err = validate_compare_chart_type("mutations", ChartType::Violin)
            .expect_err("violin should be rejected for mutation compare");
        let msg = err.to_string();
        assert!(msg.contains("study compare --type mutations"));
        assert!(msg.contains("bar"));
        assert!(msg.contains("stacked-bar"));
    }

    #[test]
    fn standalone_chart_validation_rejects_invalid_survival_chart() {
        let err = validate_standalone_chart_type(
            "study survival",
            ChartType::Histogram,
            &[ChartType::Bar, ChartType::Survival],
        )
        .expect_err("histogram should be rejected for survival");
        let msg = err.to_string();
        assert!(msg.contains("study survival"));
        assert!(msg.contains("bar"));
        assert!(msg.contains("survival"));
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
            mutation_only_caveat: None,
        };

        let svg = render_mutation_frequency_chart(&mutation, ChartType::Bar, &inline_svg_options())
            .expect("inline svg should render");

        assert!(svg.contains("<svg"));
        assert!(svg.contains("Example"));
    }

    #[test]
    fn mutation_and_cna_svg_output_use_human_readable_labels() {
        let mutation = MutationFrequencyResult {
            study_id: "demo".into(),
            gene: "TP53".into(),
            mutation_count: 7,
            unique_samples: 5,
            total_samples: 20,
            frequency: 0.25,
            top_variant_classes: vec![
                ("Missense_Mutation".into(), 4),
                ("Frame_Shift_Del".into(), 2),
                ("Splice_Site".into(), 1),
            ],
            top_protein_changes: vec![("R175H".into(), 3)],
            mutation_only_caveat: None,
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

        let mutation_svg =
            render_mutation_frequency_chart(&mutation, ChartType::Bar, &inline_svg_options())
                .expect("mutation svg");
        let cna_svg =
            render_cna_chart(&cna, ChartType::Bar, &inline_svg_options()).expect("cna svg");

        assert!(mutation_svg.contains("Missense"));
        assert!(mutation_svg.contains("Frameshift Del"));
        assert!(mutation_svg.contains("Splice"));
        assert!(!mutation_svg.contains("Missense_Mutation"));
        assert!(!mutation_svg.contains("Frame_Shift_Del"));
        assert!(!mutation_svg.contains("Splice_Site"));

        assert!(cna_svg.contains("Deep Del"));
        assert!(cna_svg.contains("Shallow Del"));
        assert!(cna_svg.contains("Diploid"));
        assert!(!cna_svg.contains("Deep deletion (-2)"));
        assert!(!cna_svg.contains("Shallow deletion (-1)"));
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
            mutation_only_caveat: None,
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
                    km_curve_points: vec![(0.0, 1.0), (12.0, 0.75), (36.0, 0.5), (60.0, 0.25)],
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
                    km_curve_points: vec![(0.0, 1.0), (12.0, 0.9), (36.0, 0.8), (60.0, 0.7)],
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

    #[test]
    fn terminal_chart_respects_custom_cols_and_rows() {
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
            mutation_only_caveat: None,
        };

        let output = render_mutation_frequency_chart(
            &mutation,
            ChartType::Bar,
            &custom_terminal_options(40, 12),
        )
        .expect("terminal chart");
        let stripped = strip_ansi(&output);
        let lines = stripped.lines().collect::<Vec<_>>();
        let widest = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);

        assert!(
            lines.len() <= 12,
            "line_count={} output={stripped}",
            lines.len()
        );
        assert!(widest <= 40, "widest={} output={stripped}", widest);
    }

    #[test]
    fn output_target_validation_rejects_incompatible_sizing_flags() {
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
            mutation_only_caveat: None,
        };

        let terminal_err = render_mutation_frequency_chart(
            &mutation,
            ChartType::Bar,
            &ChartRenderOptions {
                width: Some(1200),
                ..terminal_options()
            },
        )
        .expect_err("terminal output should reject width/height");
        assert!(
            terminal_err
                .to_string()
                .contains("--width/--height require SVG, PNG, or MCP inline SVG chart output")
        );

        let svg_err = render_mutation_frequency_chart(
            &mutation,
            ChartType::Bar,
            &ChartRenderOptions {
                cols: Some(80),
                ..svg_options(PathBuf::from("chart.svg"))
            },
        )
        .expect_err("svg output should reject terminal size flags");
        assert!(
            svg_err
                .to_string()
                .contains("--cols/--rows require terminal chart output"),
            "{svg_err}"
        );

        let inline_err = render_mutation_frequency_chart(
            &mutation,
            ChartType::Bar,
            &ChartRenderOptions {
                scale: Some(3.0),
                ..inline_svg_options()
            },
        )
        .expect_err("inline svg should reject png scale");
        assert!(
            inline_err
                .to_string()
                .contains("--scale requires PNG chart output"),
            "{inline_err}"
        );
    }

    #[test]
    fn inline_svg_output_respects_custom_dimensions() {
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
            mutation_only_caveat: None,
        };

        let svg = render_mutation_frequency_chart(
            &mutation,
            ChartType::Bar,
            &sized_inline_svg_options(1200, 600),
        )
        .expect("sized svg should render");

        assert!(svg.contains("width=\"1200"), "svg={svg}");
        assert!(svg.contains("height=\"600"), "svg={svg}");
    }

    #[test]
    fn co_occurrence_heatmap_renders_inline_svg() {
        let co_occurrence = CoOccurrenceResult {
            study_id: "demo".into(),
            genes: vec!["TP53".into(), "KRAS".into(), "PIK3CA".into()],
            total_samples: 20,
            sample_universe_basis: SampleUniverseBasis::ClinicalSampleFile,
            pairs: vec![
                CoOccurrencePair {
                    gene_a: "TP53".into(),
                    gene_b: "KRAS".into(),
                    both_mutated: 3,
                    a_only: 5,
                    b_only: 2,
                    neither: 10,
                    log_odds_ratio: Some(0.7),
                    p_value: Some(0.04),
                },
                CoOccurrencePair {
                    gene_a: "TP53".into(),
                    gene_b: "PIK3CA".into(),
                    both_mutated: 2,
                    a_only: 6,
                    b_only: 1,
                    neither: 11,
                    log_odds_ratio: Some(0.2),
                    p_value: Some(0.3),
                },
                CoOccurrencePair {
                    gene_a: "KRAS".into(),
                    gene_b: "PIK3CA".into(),
                    both_mutated: 1,
                    a_only: 4,
                    b_only: 2,
                    neither: 13,
                    log_odds_ratio: Some(-0.4),
                    p_value: Some(0.6),
                },
            ],
        };

        let svg = render_co_occurrence_chart(
            &co_occurrence,
            ChartType::Heatmap,
            &inline_svg_heatmap_options(),
        )
        .expect("co-occurrence heatmap svg");

        assert!(svg.contains("<svg"));
        assert!(svg.contains("Co-mutation heatmap (demo)"));
        assert!(svg.contains("TP53"));
        assert!(svg.contains("KRAS"));
        assert!(svg.contains("PIK3CA"));
    }

    #[test]
    fn co_occurrence_heatmap_respects_custom_dimensions() {
        let co_occurrence = CoOccurrenceResult {
            study_id: "demo".into(),
            genes: vec!["TP53".into(), "KRAS".into(), "PIK3CA".into()],
            total_samples: 20,
            sample_universe_basis: SampleUniverseBasis::ClinicalSampleFile,
            pairs: vec![
                CoOccurrencePair {
                    gene_a: "TP53".into(),
                    gene_b: "KRAS".into(),
                    both_mutated: 3,
                    a_only: 5,
                    b_only: 2,
                    neither: 10,
                    log_odds_ratio: Some(0.7),
                    p_value: Some(0.04),
                },
                CoOccurrencePair {
                    gene_a: "TP53".into(),
                    gene_b: "PIK3CA".into(),
                    both_mutated: 2,
                    a_only: 6,
                    b_only: 1,
                    neither: 11,
                    log_odds_ratio: Some(0.2),
                    p_value: Some(0.3),
                },
                CoOccurrencePair {
                    gene_a: "KRAS".into(),
                    gene_b: "PIK3CA".into(),
                    both_mutated: 1,
                    a_only: 4,
                    b_only: 2,
                    neither: 13,
                    log_odds_ratio: Some(-0.4),
                    p_value: Some(0.6),
                },
            ],
        };

        let svg = render_co_occurrence_chart(
            &co_occurrence,
            ChartType::Heatmap,
            &ChartRenderOptions {
                width: Some(900),
                height: Some(500),
                ..inline_svg_heatmap_options()
            },
        )
        .expect("co-occurrence heatmap svg");

        assert!(svg.contains("width=\"900"), "svg={svg}");
        assert!(svg.contains("height=\"500"), "svg={svg}");
    }

    #[test]
    fn co_occurrence_heatmap_rejects_palette_override() {
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

        let err =
            render_co_occurrence_chart(&co_occurrence, ChartType::Heatmap, &inline_svg_options())
                .expect_err("heatmap should reject palette overrides");
        assert!(err.to_string().contains(
            "--palette is not supported for 'study co-occurrence --chart heatmap'. Omit --palette; heatmaps use a fixed continuous colormap."
        ));
    }

    #[test]
    fn mutation_compare_stacked_bar_renders_inline_svg() {
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

        let svg = render_mutation_compare_chart(
            &mutation_compare,
            ChartType::StackedBar,
            &inline_svg_auto_title_options(),
        )
        .expect("stacked-bar svg");

        assert!(svg.contains("<svg"));
        assert!(svg.contains("PIK3CA mutation status by TP53"));
        assert!(svg.contains("Mutated"));
        assert!(svg.contains("Not mutated"));
        assert!(svg.contains("Samples"));
    }

    #[test]
    fn mutation_compare_stacked_bar_uses_integer_sample_ticks() {
        let mutation_compare = MutationComparisonResult {
            study_id: "demo".into(),
            stratify_gene: "TP53".into(),
            target_gene: "PIK3CA".into(),
            groups: vec![
                MutationGroupStats {
                    group_name: "TP53-mutant".into(),
                    sample_count: 2,
                    mutated_count: 0,
                    mutation_rate: 0.0,
                },
                MutationGroupStats {
                    group_name: "TP53-wildtype".into(),
                    sample_count: 1,
                    mutated_count: 1,
                    mutation_rate: 1.0,
                },
            ],
        };

        let svg = render_mutation_compare_chart(
            &mutation_compare,
            ChartType::StackedBar,
            &inline_svg_auto_title_options(),
        )
        .expect("stacked-bar svg");

        assert!(!svg.contains(">0.5<"), "svg={svg}");
        assert!(!svg.contains(">1.5<"), "svg={svg}");
    }

    #[test]
    fn mutation_waterfall_renders_inline_svg() {
        let svg = render_mutation_waterfall_chart(
            "demo",
            "TP53",
            &[
                ("S3".to_string(), 4),
                ("S1".to_string(), 2),
                ("S2".to_string(), 1),
            ],
            &inline_svg_auto_title_options(),
        )
        .expect("waterfall svg");

        assert!(svg.contains("<svg"));
        assert!(svg.contains("TP53 mutation burden by sample (demo)"));
        assert!(svg.contains("S3"));
        assert!(svg.contains("S1"));
        assert!(svg.contains("S2"));
    }

    #[test]
    fn expression_scatter_renders_inline_svg_and_rejects_empty_points() {
        let svg = render_expression_scatter_chart(
            "demo",
            "TP53",
            "ERBB2",
            &[(1.0, 2.0), (2.0, 4.0), (3.0, 1.0)],
            &inline_svg_auto_title_options(),
        )
        .expect("scatter svg");

        assert!(svg.contains("<svg"));
        assert!(svg.contains("TP53 vs ERBB2 expression (demo)"));
        assert!(svg.contains("TP53"));
        assert!(svg.contains("ERBB2"));
        assert!(!svg.contains("r = "), "svg={svg}");

        let err = render_expression_scatter_chart(
            "demo",
            "TP53",
            "ERBB2",
            &[],
            &inline_svg_auto_title_options(),
        )
        .expect_err("scatter should reject empty points");
        assert!(
            err.to_string()
                .contains("scatter chart requires at least one paired sample")
        );
    }

    #[test]
    fn heatmap_and_stacked_bar_svg_outputs_write_files() {
        let output_dir = TestOutputDir::new();
        let heatmap = CoOccurrenceResult {
            study_id: "demo".into(),
            genes: vec!["TP53".into(), "KRAS".into(), "PIK3CA".into()],
            total_samples: 20,
            sample_universe_basis: SampleUniverseBasis::ClinicalSampleFile,
            pairs: vec![
                CoOccurrencePair {
                    gene_a: "TP53".into(),
                    gene_b: "KRAS".into(),
                    both_mutated: 3,
                    a_only: 5,
                    b_only: 2,
                    neither: 10,
                    log_odds_ratio: Some(0.7),
                    p_value: Some(0.04),
                },
                CoOccurrencePair {
                    gene_a: "TP53".into(),
                    gene_b: "PIK3CA".into(),
                    both_mutated: 2,
                    a_only: 6,
                    b_only: 1,
                    neither: 11,
                    log_odds_ratio: Some(0.2),
                    p_value: Some(0.3),
                },
                CoOccurrencePair {
                    gene_a: "KRAS".into(),
                    gene_b: "PIK3CA".into(),
                    both_mutated: 1,
                    a_only: 4,
                    b_only: 2,
                    neither: 13,
                    log_odds_ratio: Some(-0.4),
                    p_value: Some(0.6),
                },
            ],
        };
        let mutation_compare = MutationComparisonResult {
            study_id: "demo".into(),
            stratify_gene: "TP53".into(),
            target_gene: "PIK3CA".into(),
            groups: vec![
                MutationGroupStats {
                    group_name: "TP53-mutant".into(),
                    sample_count: 2,
                    mutated_count: 0,
                    mutation_rate: 0.0,
                },
                MutationGroupStats {
                    group_name: "TP53-wildtype".into(),
                    sample_count: 1,
                    mutated_count: 1,
                    mutation_rate: 1.0,
                },
            ],
        };
        let heatmap_path = output_dir.svg_path("heatmap.svg");
        let stacked_path = output_dir.svg_path("stacked.svg");

        assert!(
            render_co_occurrence_chart(
                &heatmap,
                ChartType::Heatmap,
                &svg_heatmap_options(heatmap_path.clone()),
            )
            .expect("heatmap svg")
            .contains(heatmap_path.to_string_lossy().as_ref())
        );
        assert!(
            render_mutation_compare_chart(
                &mutation_compare,
                ChartType::StackedBar,
                &svg_options(stacked_path.clone()),
            )
            .expect("stacked-bar svg")
            .contains(stacked_path.to_string_lossy().as_ref())
        );

        let heatmap_svg = fs::read_to_string(&heatmap_path).expect("heatmap file should exist");
        let stacked_svg = fs::read_to_string(&stacked_path).expect("stacked file should exist");
        assert!(heatmap_svg.contains("<svg"));
        assert!(stacked_svg.contains("<svg"));
    }

    #[test]
    fn survival_svg_output_supports_kaplan_meier_curves() {
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
                    km_curve_points: vec![(0.0, 1.0), (12.0, 0.75), (36.0, 0.5), (60.0, 0.25)],
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
                    km_curve_points: vec![(0.0, 1.0), (12.0, 0.9), (36.0, 0.8), (60.0, 0.7)],
                },
            ],
            log_rank_p: Some(0.02),
        };

        let svg = render_survival_chart(
            &survival,
            ChartType::Survival,
            &inline_svg_auto_title_options(),
        )
        .expect("survival svg");

        assert!(svg.contains("Time (months)"));
        assert!(svg.contains("Survival probability"));
        assert!(svg.contains("TP53-mutant"));
        assert!(svg.contains("TP53-wildtype"));
        assert!(svg.contains("Overall Survival"));
    }
}
