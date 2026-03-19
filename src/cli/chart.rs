use std::borrow::Cow;

use clap::Subcommand;
use rust_embed::RustEmbed;

use crate::error::BioMcpError;

#[derive(RustEmbed)]
#[folder = "docs/charts/"]
struct EmbeddedCharts;

#[derive(Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartCommand {
    Bar,
    Pie,
    Histogram,
    Density,
    Box,
    Violin,
    Ridgeline,
    Survival,
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
        Some(ChartCommand::Pie) => "pie.md",
        Some(ChartCommand::Histogram) => "histogram.md",
        Some(ChartCommand::Density) => "density.md",
        Some(ChartCommand::Box) => "box.md",
        Some(ChartCommand::Violin) => "violin.md",
        Some(ChartCommand::Ridgeline) => "ridgeline.md",
        Some(ChartCommand::Survival) => "survival.md",
    };
    embedded_text(path)
}
