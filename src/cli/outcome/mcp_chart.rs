//! MCP chart argument checks and rewriting, split out of the outcome seam.
use super::mcp_output_flag_error;
use crate::cli::{Cli, Commands, StudyCommand};

pub(super) fn is_charted_mcp_study_command(cli: &Cli) -> Result<bool, crate::error::BioMcpError> {
    let chart = match &cli.command {
        Commands::Study {
            cmd:
                StudyCommand::Query { chart, .. }
                | StudyCommand::Survival { chart, .. }
                | StudyCommand::Compare { chart, .. }
                | StudyCommand::CoOccurrence { chart, .. },
        } => chart,
        _ => return Ok(false),
    };

    if chart.chart.is_none() || cli.json {
        return Ok(false);
    }
    if chart.output.is_some() {
        return Err(mcp_output_flag_error());
    }
    Ok(true)
}

pub(super) fn prepare_mcp_chart(cli: &mut Cli) -> Result<(), crate::error::BioMcpError> {
    let chart = match &mut cli.command {
        Commands::Study {
            cmd:
                StudyCommand::Query { chart, .. }
                | StudyCommand::Survival { chart, .. }
                | StudyCommand::Compare { chart, .. }
                | StudyCommand::CoOccurrence { chart, .. },
        } => chart,
        _ => return Ok(()),
    };
    if chart.chart.is_none() || cli.json {
        return Ok(());
    }
    if chart.output.is_some() {
        return Err(mcp_output_flag_error());
    }
    if chart.cols.is_some() || chart.rows.is_some() {
        return Err(crate::error::BioMcpError::InvalidArgument(
            crate::render::chart::TERMINAL_SIZE_FLAGS_ERROR.into(),
        ));
    }
    if chart.scale.is_some() {
        return Err(crate::error::BioMcpError::InvalidArgument(
            crate::render::chart::PNG_SCALE_FLAGS_ERROR.into(),
        ));
    }
    chart.mcp_inline = true;
    Ok(())
}
