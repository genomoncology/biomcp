use std::borrow::Cow;

use clap::Subcommand;
use rust_embed::RustEmbed;

use crate::error::BioMcpError;

#[derive(RustEmbed)]
#[folder = "docs/charts/"]
struct EmbeddedCharts;

#[derive(Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartCommand {
    /// Categorical counts as vertical bars
    Bar,
    /// Mutation-grouped sample counts split by outcome
    StackedBar,
    /// Proportional distribution of categories
    Pie,
    /// Ranked per-sample mutation burden
    Waterfall,
    /// Pairwise co-occurrence matrix
    Heatmap,
    /// Binned distribution of a continuous value
    Histogram,
    /// Smoothed distribution estimate
    Density,
    /// Median, IQR, and whiskers for group comparison
    Box,
    /// Full distribution shape for group comparison
    Violin,
    /// Stacked density comparison across groups
    Ridgeline,
    /// Paired expression values for two genes
    Scatter,
    /// Kaplan-Meier survival curves
    Survival,
}

impl ChartCommand {
    pub const fn as_str(self) -> &'static str {
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

impl std::fmt::Display for ChartCommand {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

fn embedded_text(path: &str) -> Result<String, BioMcpError> {
    let Some(asset) = EmbeddedCharts::get(path) else {
        return Err(BioMcpError::NotFound {
            entity: "chart".into(),
            id: path.to_string(),
            suggestion: "Try: biomcp chart".into(),
        });
    };
    let bytes: Cow<'static, [u8]> = asset.data;
    String::from_utf8(bytes.into_owned())
        .map_err(|_| BioMcpError::InvalidArgument("Embedded chart doc is not valid UTF-8".into()))
}

pub fn show(command: Option<&ChartCommand>) -> Result<String, BioMcpError> {
    let path = match command {
        None => "index.md",
        Some(ChartCommand::Bar) => "bar.md",
        Some(ChartCommand::StackedBar) => "stacked-bar.md",
        Some(ChartCommand::Pie) => "pie.md",
        Some(ChartCommand::Waterfall) => "waterfall.md",
        Some(ChartCommand::Heatmap) => "heatmap.md",
        Some(ChartCommand::Histogram) => "histogram.md",
        Some(ChartCommand::Density) => "density.md",
        Some(ChartCommand::Box) => "box.md",
        Some(ChartCommand::Violin) => "violin.md",
        Some(ChartCommand::Ridgeline) => "ridgeline.md",
        Some(ChartCommand::Scatter) => "scatter.md",
        Some(ChartCommand::Survival) => "survival.md",
    };
    embedded_text(path)
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser};

    use super::{ChartCommand, show};
    use crate::cli::{Cli, Commands};

    fn render_chart_long_help() -> String {
        let mut command = Cli::command();
        let chart = command
            .find_subcommand_mut("chart")
            .expect("chart subcommand should exist");
        let mut help = Vec::new();
        chart
            .write_long_help(&mut help)
            .expect("chart help should render");
        String::from_utf8(help).expect("help should be utf-8")
    }

    #[test]
    fn chart_help_lists_descriptions_for_all_chart_topics() {
        let help = render_chart_long_help();

        assert!(help.contains("bar          Categorical counts as vertical bars"));
        assert!(help.contains("stacked-bar  Mutation-grouped sample counts split by outcome"));
        assert!(help.contains("pie          Proportional distribution of categories"));
        assert!(help.contains("waterfall    Ranked per-sample mutation burden"));
        assert!(help.contains("heatmap      Pairwise co-occurrence matrix"));
        assert!(help.contains("histogram    Binned distribution of a continuous value"));
        assert!(help.contains("density      Smoothed distribution estimate"));
        assert!(help.contains("box          Median, IQR, and whiskers for group comparison"));
        assert!(help.contains("violin       Full distribution shape for group comparison"));
        assert!(help.contains("ridgeline    Stacked density comparison across groups"));
        assert!(help.contains("scatter      Paired expression values for two genes"));
        assert!(help.contains("survival     Kaplan-Meier survival curves"));
    }

    #[test]
    fn chart_subcommand_parses_violin_topic() {
        let cli =
            Cli::try_parse_from(["biomcp", "chart", "violin"]).expect("chart docs should parse");

        assert!(matches!(
            cli.command,
            Commands::Chart {
                command: Some(ChartCommand::Violin),
            }
        ));
    }

    #[test]
    fn chart_subcommand_parses_heatmap_topic() {
        let cli = Cli::try_parse_from(["biomcp", "chart", "heatmap"])
            .expect("heatmap chart docs should parse");

        assert!(matches!(
            cli.command,
            Commands::Chart {
                command: Some(ChartCommand::Heatmap),
            }
        ));
    }

    #[test]
    fn chart_subcommand_parses_waterfall_topic() {
        let cli = Cli::try_parse_from(["biomcp", "chart", "waterfall"])
            .expect("waterfall chart docs should parse");

        assert!(matches!(
            cli.command,
            Commands::Chart {
                command: Some(ChartCommand::Waterfall),
            }
        ));
    }

    #[test]
    fn chart_subcommand_parses_scatter_topic() {
        let cli = Cli::try_parse_from(["biomcp", "chart", "scatter"])
            .expect("scatter chart docs should parse");

        assert!(matches!(
            cli.command,
            Commands::Chart {
                command: Some(ChartCommand::Scatter),
            }
        ));
    }

    #[test]
    fn chart_subcommand_parses_stacked_bar_topic() {
        let cli = Cli::try_parse_from(["biomcp", "chart", "stacked-bar"])
            .expect("stacked-bar chart docs should parse");

        assert!(matches!(
            cli.command,
            Commands::Chart {
                command: Some(ChartCommand::StackedBar),
            }
        ));
    }

    #[test]
    fn chart_subcommand_parses_survival_topic() {
        let cli = Cli::try_parse_from(["biomcp", "chart", "survival"])
            .expect("survival chart docs should parse");

        assert!(matches!(
            cli.command,
            Commands::Chart {
                command: Some(ChartCommand::Survival),
            }
        ));
    }

    #[test]
    fn show_returns_heatmap_doc() {
        let doc = show(Some(&ChartCommand::Heatmap)).expect("heatmap doc should exist");
        assert!(doc.contains("# Heatmap"));
        assert!(doc.contains("study co-occurrence --chart heatmap"));
    }

    #[test]
    fn show_returns_stacked_bar_doc() {
        let doc = show(Some(&ChartCommand::StackedBar)).expect("stacked-bar doc should exist");
        assert!(doc.contains("# Stacked Bar Chart"));
        assert!(doc.contains("study compare --type mutations --chart stacked-bar"));
    }

    #[test]
    fn show_returns_waterfall_doc() {
        let doc = show(Some(&ChartCommand::Waterfall)).expect("waterfall doc should exist");
        assert!(doc.contains("# Waterfall"));
        assert!(doc.contains("study query --type mutations --chart waterfall"));
    }

    #[test]
    fn show_returns_scatter_doc() {
        let doc = show(Some(&ChartCommand::Scatter)).expect("scatter doc should exist");
        assert!(doc.contains("# Scatter"));
        assert!(doc.contains("study compare --type expression --chart scatter"));
    }
}
