//! Shared CLI-facing types and chart argument parsing used by the stable CLI facade.

use std::path::PathBuf;

use clap::{Args, Parser, ValueEnum};

use crate::entities::drug::DrugRegion;

#[derive(Parser, Debug)]
#[command(
    name = "biomcp",
    about = "Query genes, variants, trials, articles, drugs, diseases, and more from leading public biomedical data sources",
    after_help = "Note: flags marked (best-effort) are applied client-side or via imprecise API matching; results may include false positives."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: super::Commands,

    /// Output as JSON instead of Markdown (except biomcp cache path, which stays plain text)
    #[arg(short, long, global = true)]
    pub json: bool,

    /// Use no managed request state: bypass HTTP cache and article sessions
    #[arg(long, global = true)]
    pub no_cache: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ChartType {
    Bar,
    StackedBar,
    Pie,
    Waterfall,
    Heatmap,
    Histogram,
    Density,
    Box,
    Violin,
    Ridgeline,
    Scatter,
    Survival,
}

impl ChartType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bar => "bar",
            Self::StackedBar => "stacked-bar",
            Self::Pie => "pie",
            Self::Waterfall => "waterfall",
            Self::Heatmap => "heatmap",
            Self::Histogram => "histogram",
            Self::Density => "density",
            Self::Box => "box",
            Self::Violin => "violin",
            Self::Ridgeline => "ridgeline",
            Self::Scatter => "scatter",
            Self::Survival => "survival",
        }
    }
}

impl std::fmt::Display for ChartType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

fn parse_chart_positive_usize(flag: &str, value: &str) -> Result<usize, String> {
    let parsed = value
        .trim()
        .parse::<usize>()
        .map_err(|_| format!("{flag} must be an integer >= 1"))?;
    if parsed == 0 {
        return Err(format!("{flag} must be >= 1"));
    }
    Ok(parsed)
}

fn parse_chart_positive_u32(flag: &str, value: &str) -> Result<u32, String> {
    let parsed = value
        .trim()
        .parse::<u32>()
        .map_err(|_| format!("{flag} must be an integer >= 1"))?;
    if parsed == 0 {
        return Err(format!("{flag} must be >= 1"));
    }
    Ok(parsed)
}

fn parse_chart_cols(value: &str) -> Result<usize, String> {
    parse_chart_positive_usize("--cols", value)
}

fn parse_chart_rows(value: &str) -> Result<usize, String> {
    parse_chart_positive_usize("--rows", value)
}

fn parse_chart_width(value: &str) -> Result<u32, String> {
    parse_chart_positive_u32("--width", value)
}

fn parse_chart_height(value: &str) -> Result<u32, String> {
    parse_chart_positive_u32("--height", value)
}

fn parse_chart_scale(value: &str) -> Result<f32, String> {
    let parsed = value
        .trim()
        .parse::<f32>()
        .map_err(|_| "--scale must be a finite number > 0".to_string())?;
    if !parsed.is_finite() {
        return Err("--scale must be a finite number > 0".to_string());
    }
    if parsed <= 0.0 {
        return Err("--scale must be > 0".to_string());
    }
    Ok(parsed)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DrugRegionArg {
    Us,
    #[value(alias = "ema")]
    Eu,
    Who,
    All,
}

impl From<DrugRegionArg> for DrugRegion {
    fn from(value: DrugRegionArg) -> Self {
        match value {
            DrugRegionArg::Us => DrugRegion::Us,
            DrugRegionArg::Eu => DrugRegion::Eu,
            DrugRegionArg::Who => DrugRegion::Who,
            DrugRegionArg::All => DrugRegion::All,
        }
    }
}

#[derive(Args, Debug, Clone, PartialEq, Default)]
pub struct ChartArgs {
    #[arg(
        long,
        value_enum,
        help = "Render a chart instead of standard study output",
        hide_short_help = true,
        help_heading = "Chart Output"
    )]
    pub chart: Option<ChartType>,

    #[arg(
        long,
        requires = "chart",
        conflicts_with = "output",
        help = "Render the chart in the terminal",
        hide_short_help = true,
        help_heading = "Chart Output"
    )]
    pub terminal: bool,

    #[arg(
        short = 'o',
        long = "output",
        value_name = "FILE",
        requires = "chart",
        help = "Write the chart to FILE (.svg or .png)",
        hide_short_help = true,
        help_heading = "Chart Output"
    )]
    pub output: Option<PathBuf>,

    #[arg(
        long,
        requires = "chart",
        help = "Override the auto-generated chart title",
        hide_short_help = true,
        help_heading = "Chart Styling"
    )]
    pub title: Option<String>,

    #[arg(
        long,
        requires = "chart",
        help = "Set chart theme: light, dark, solarized, or minimal",
        hide_short_help = true,
        help_heading = "Chart Styling"
    )]
    pub theme: Option<String>,

    #[arg(
        long,
        requires = "chart",
        help = "Set categorical chart palette",
        hide_short_help = true,
        help_heading = "Chart Styling"
    )]
    pub palette: Option<String>,

    #[arg(
        long,
        value_name = "N",
        value_parser = parse_chart_cols,
        requires = "chart",
        help = "Terminal chart width in character cells",
        hide_short_help = true,
        help_heading = "Chart Styling"
    )]
    pub cols: Option<usize>,

    #[arg(
        long,
        value_name = "N",
        value_parser = parse_chart_rows,
        requires = "chart",
        help = "Terminal chart height in character cells",
        hide_short_help = true,
        help_heading = "Chart Styling"
    )]
    pub rows: Option<usize>,

    #[arg(
        long,
        value_name = "PX",
        value_parser = parse_chart_width,
        requires = "chart",
        help = "Chart canvas width in pixels for SVG, PNG, or MCP inline SVG",
        hide_short_help = true,
        help_heading = "Chart Styling"
    )]
    pub width: Option<u32>,

    #[arg(
        long,
        value_name = "PX",
        value_parser = parse_chart_height,
        requires = "chart",
        help = "Chart canvas height in pixels for SVG, PNG, or MCP inline SVG",
        hide_short_help = true,
        help_heading = "Chart Styling"
    )]
    pub height: Option<u32>,

    #[arg(
        long,
        value_name = "FACTOR",
        value_parser = parse_chart_scale,
        requires = "chart",
        help = "PNG pixel-density multiplier",
        hide_short_help = true,
        help_heading = "Chart Styling"
    )]
    pub scale: Option<f32>,

    #[arg(long, hide = true, requires = "chart")]
    pub mcp_inline: bool,
}

#[derive(Debug)]
pub struct CliOutput {
    pub text: String,
    pub metadata_json: Option<String>,
    pub svg: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone)]
pub struct CommandOutcome {
    pub text: String,
    pub metadata_json: Option<String>,
    pub bytes: Option<Vec<u8>>,
    pub svg: Option<String>,
    pub stream: OutputStream,
    pub exit_code: u8,
}

impl CommandOutcome {
    pub(crate) fn stdout(text: String) -> Self {
        Self {
            text,
            metadata_json: None,
            bytes: None,
            svg: None,
            stream: OutputStream::Stdout,
            exit_code: 0,
        }
    }

    pub(crate) fn stdout_bytes(bytes: Vec<u8>) -> Self {
        Self {
            text: String::new(),
            metadata_json: None,
            bytes: Some(bytes),
            svg: None,
            stream: OutputStream::Stdout,
            exit_code: 0,
        }
    }

    pub(crate) fn stdout_with_exit(text: String, exit_code: u8) -> Self {
        Self {
            text,
            metadata_json: None,
            bytes: None,
            svg: None,
            stream: OutputStream::Stdout,
            exit_code,
        }
    }

    pub(crate) fn stderr_with_exit(text: String, exit_code: u8) -> Self {
        Self {
            text,
            metadata_json: None,
            bytes: None,
            svg: None,
            stream: OutputStream::Stderr,
            exit_code,
        }
    }

    pub(crate) fn with_metadata_json(mut self, metadata_json: String) -> Self {
        self.metadata_json = Some(metadata_json);
        self
    }

    pub(crate) fn stdout_with_svg(text: String, svg: String) -> Self {
        Self {
            text,
            metadata_json: None,
            bytes: None,
            svg: Some(svg),
            stream: OutputStream::Stdout,
            exit_code: 0,
        }
    }
}
