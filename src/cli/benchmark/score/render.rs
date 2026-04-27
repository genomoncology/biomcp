//! Session score markdown rendering.

use super::super::types::SessionScoreReport;

fn render_human_report(report: &SessionScoreReport, brief: bool) -> String {
    let mut out = String::new();
    out.push_str("# Session Score\n\n");
    out.push_str(&format!("- Session: {}\n", report.session_path));
    out.push_str(&format!("- Tool calls: {}\n", report.total_tool_calls));
    out.push_str(&format!("- BioMCP commands: {}\n", report.biomcp_commands));
    out.push_str(&format!("- Help calls: {}\n", report.help_calls));
    out.push_str(&format!("- Skill reads: {}\n", report.skill_reads));
    out.push_str(&format!("- Errors: {}\n", report.errors_total));
    out.push_str(&format!(
        "- Error categories: ghost={} quoting={} api={} other={}\n",
        report.error_categories.ghost,
        report.error_categories.quoting,
        report.error_categories.api,
        report.error_categories.other,
    ));

    if let Some(ms) = report.wall_time_ms {
        out.push_str(&format!("- Wall time: {} ms\n", ms));
    }

    out.push_str(&format!(
        "- Tokens: input={} output={} cache_read={} cache_write={} cost_usd={:.6}\n",
        report.tokens.input_tokens,
        report.tokens.output_tokens,
        report.tokens.cache_read_tokens,
        report.tokens.cache_write_tokens,
        report.tokens.cost_usd,
    ));

    if let Some(coverage) = &report.coverage {
        out.push_str("\n## Coverage\n\n");
        out.push_str(&format!(
            "- expected={} hits={} misses={} extras={}\n",
            coverage.expected_total, coverage.hits, coverage.misses, coverage.extras,
        ));

        if !brief {
            if !coverage.missing_commands.is_empty() {
                out.push_str("\nMissing commands:\n");
                for command in &coverage.missing_commands {
                    out.push_str(&format!("- {}\n", command));
                }
            }

            if !coverage.extra_commands.is_empty() {
                out.push_str("\nExtra commands:\n");
                for command in &coverage.extra_commands {
                    out.push_str(&format!("- {}\n", command));
                }
            }
        }
    }

    if !brief && !report.command_shapes.is_empty() {
        out.push_str("\n## Command Shapes\n\n");
        for shape in &report.command_shapes {
            out.push_str(&format!("- {}\n", shape));
        }
    }

    out
}

#[cfg(test)]
#[path = "tests/render.rs"]
mod tests;
