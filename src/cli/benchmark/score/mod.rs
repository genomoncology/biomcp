//! Benchmark session scoring facade.

use std::collections::BTreeSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use anyhow::Context;
use serde_json::Value;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use super::types::{
    SESSION_SCORE_SCHEMA_VERSION, SessionErrorCategories, SessionScoreReport, SessionTokenUsage,
};

mod normalize;
mod parse;
mod render;

use normalize::{compute_coverage, normalize_command_shape};
use parse::{
    ErrorCategory, classify_error, collapse_whitespace, collect_error_messages,
    collect_token_usage, collect_tool_calls, extract_timestamp, is_biomcp_shell_tool,
    is_help_command, is_skill_read_command,
};
use render::render_human_report;

#[derive(Debug, Clone)]
pub struct ScoreSessionOptions {
    pub session: PathBuf,
    pub expected: Option<PathBuf>,
    pub brief: bool,
}

#[derive(Debug, Default)]
struct ScoreAccumulator {
    total_tool_calls: u64,
    biomcp_commands: u64,
    help_calls: u64,
    skill_reads: u64,
    errors_total: u64,
    error_categories: SessionErrorCategories,
    tokens: SessionTokenUsage,
    command_shapes: BTreeSet<String>,
    observed_shapes: BTreeSet<String>,
    first_timestamp: Option<OffsetDateTime>,
    last_timestamp: Option<OffsetDateTime>,
}

pub fn score_session(opts: ScoreSessionOptions, json_output: bool) -> anyhow::Result<String> {
    let report = score_session_file(&opts)?;

    if json_output {
        return Ok(crate::render::json::to_pretty(&report)?);
    }

    Ok(render_human_report(&report, opts.brief))
}

fn score_session_file(opts: &ScoreSessionOptions) -> anyhow::Result<SessionScoreReport> {
    let file = fs::File::open(&opts.session)
        .with_context(|| format!("failed to open session file {}", opts.session.display()))?;
    let reader = BufReader::new(file);

    let mut acc = ScoreAccumulator::default();

    for line_result in reader.lines() {
        let line = line_result.context("failed to read jsonl line")?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let value = serde_json::from_str::<Value>(line)
            .with_context(|| format!("invalid JSONL line: {line}"))?;

        if let Some(timestamp) = extract_timestamp(&value) {
            acc.first_timestamp = match acc.first_timestamp {
                Some(existing) if existing <= timestamp => Some(existing),
                _ => Some(timestamp),
            };
            acc.last_timestamp = match acc.last_timestamp {
                Some(existing) if existing >= timestamp => Some(existing),
                _ => Some(timestamp),
            };
        }

        let mut tool_calls = Vec::new();
        collect_tool_calls(&value, &mut tool_calls);
        if !tool_calls.is_empty() {
            acc.total_tool_calls += tool_calls.len() as u64;
        }

        for call in tool_calls {
            let collapsed = collapse_whitespace(&call.command);

            if is_skill_read_command(&collapsed) {
                acc.skill_reads += 1;
            }

            if let Some(shape) = normalize_command_shape(&collapsed) {
                if is_biomcp_shell_tool(&call.tool_name) {
                    acc.biomcp_commands += 1;
                }
                if is_help_command(&collapsed) {
                    acc.help_calls += 1;
                }
                acc.command_shapes.insert(shape.clone());
                acc.observed_shapes.insert(shape);
            }
        }

        let mut errors = BTreeSet::new();
        collect_error_messages(&value, &mut errors);
        for err in errors {
            acc.errors_total += 1;
            match classify_error(&err) {
                ErrorCategory::Ghost => acc.error_categories.ghost += 1,
                ErrorCategory::Quoting => acc.error_categories.quoting += 1,
                ErrorCategory::Api => acc.error_categories.api += 1,
                ErrorCategory::Other => acc.error_categories.other += 1,
            }
        }

        collect_token_usage(&value, &mut acc.tokens);
    }

    let coverage = if let Some(expected_path) = &opts.expected {
        Some(compute_coverage(expected_path, &acc.observed_shapes)?)
    } else {
        None
    };

    let wall_time_ms = if let (Some(first), Some(last)) = (acc.first_timestamp, acc.last_timestamp)
    {
        if last >= first {
            let duration = last - first;
            Some(duration.whole_milliseconds() as u64)
        } else {
            None
        }
    } else {
        None
    };

    Ok(SessionScoreReport {
        schema_version: SESSION_SCORE_SCHEMA_VERSION,
        generated_at: now_rfc3339()?,
        session_path: opts.session.display().to_string(),
        total_tool_calls: acc.total_tool_calls,
        biomcp_commands: acc.biomcp_commands,
        help_calls: acc.help_calls,
        skill_reads: acc.skill_reads,
        errors_total: acc.errors_total,
        error_categories: acc.error_categories,
        coverage,
        tokens: acc.tokens,
        wall_time_ms,
        command_shapes: acc.command_shapes.into_iter().collect(),
    })
}

fn now_rfc3339() -> anyhow::Result<String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .context("failed to format score timestamp")
}

#[cfg(test)]
#[path = "tests/parse.rs"]
mod parse_tests;
