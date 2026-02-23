use std::collections::BTreeSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde_json::{Map, Value};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use super::types::{
    SESSION_SCORE_SCHEMA_VERSION, SessionCoverage, SessionErrorCategories, SessionScoreReport,
    SessionTokenUsage,
};

#[derive(Debug, Clone)]
pub struct ScoreSessionOptions {
    pub session: PathBuf,
    pub expected: Option<PathBuf>,
    pub brief: bool,
}

#[derive(Debug)]
struct ToolCall {
    tool_name: String,
    command: String,
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

fn compute_coverage(
    path: &Path,
    observed_shapes: &BTreeSet<String>,
) -> anyhow::Result<SessionCoverage> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read expected commands file {}", path.display()))?;

    let mut expected_shapes = BTreeSet::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(shape) = normalize_command_shape(trimmed) {
            expected_shapes.insert(shape);
        }
    }

    let hits = expected_shapes
        .iter()
        .filter(|shape| observed_shapes.contains(*shape))
        .count();

    let missing_commands = expected_shapes
        .iter()
        .filter(|shape| !observed_shapes.contains(*shape))
        .cloned()
        .collect::<Vec<_>>();

    let extra_commands = observed_shapes
        .iter()
        .filter(|shape| !expected_shapes.contains(*shape))
        .cloned()
        .collect::<Vec<_>>();

    Ok(SessionCoverage {
        expected_total: expected_shapes.len(),
        hits,
        misses: missing_commands.len(),
        extras: extra_commands.len(),
        missing_commands,
        extra_commands,
    })
}

fn collect_tool_calls(value: &Value, out: &mut Vec<ToolCall>) {
    match value {
        Value::Object(map) => {
            if let Some(name) = extract_tool_name(map)
                && let Some(command) = extract_command_from_object(map)
                && is_shell_tool_name(&name)
            {
                out.push(ToolCall {
                    tool_name: name,
                    command,
                });
            }

            if let Some(function) = map.get("function").and_then(Value::as_object)
                && let Some(name) = extract_tool_name(function)
                && let Some(command) = extract_command_from_object(function)
                && is_shell_tool_name(&name)
            {
                out.push(ToolCall {
                    tool_name: name,
                    command,
                });
            }

            for nested in map.values() {
                collect_tool_calls(nested, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_tool_calls(item, out);
            }
        }
        _ => {}
    }
}

fn extract_tool_name(map: &Map<String, Value>) -> Option<String> {
    for key in ["tool_name", "name", "tool"] {
        if let Some(value) = map.get(key).and_then(Value::as_str) {
            return Some(value.to_string());
        }
    }
    None
}

fn extract_command_from_object(map: &Map<String, Value>) -> Option<String> {
    for key in ["cmd", "command"] {
        if let Some(cmd) = map.get(key).and_then(Value::as_str) {
            return Some(cmd.to_string());
        }
    }

    for key in ["input", "arguments", "args", "payload", "params"] {
        if let Some(value) = map.get(key)
            && let Some(command) = extract_command_from_value(value)
        {
            return Some(command);
        }
    }

    None
}

fn extract_command_from_value(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => {
            if text.contains("biomcp") || text.contains("skills/") {
                return Some(text.to_string());
            }

            if let Ok(parsed) = serde_json::from_str::<Value>(text) {
                return extract_command_from_value(&parsed);
            }

            None
        }
        Value::Object(map) => extract_command_from_object(map),
        Value::Array(items) => {
            for item in items {
                if let Some(command) = extract_command_from_value(item) {
                    return Some(command);
                }
            }
            None
        }
        _ => None,
    }
}

fn is_shell_tool_name(name: &str) -> bool {
    let normalized = name.trim().to_ascii_lowercase();
    normalized == "bash"
        || normalized == "shell"
        || normalized.ends_with(".bash")
        || normalized.ends_with(".shell")
}

fn is_biomcp_shell_tool(name: &str) -> bool {
    is_shell_tool_name(name)
}

fn normalize_command_shape(command: &str) -> Option<String> {
    let tokens = shlex::split(command)?;
    if tokens.is_empty() {
        return None;
    }

    let mut start = None;
    for (idx, token) in tokens.iter().enumerate() {
        if is_biomcp_binary_token(token) {
            start = Some(idx);
            break;
        }
    }

    let start = if let Some(idx) = start {
        idx
    } else {
        for token in &tokens {
            if let Some(extracted) = extract_embedded_biomcp(token)
                && extracted != command
            {
                return normalize_command_shape(&extracted);
            }
        }
        return None;
    };
    let mut normalized = Vec::new();
    let mut positional_index = 0usize;
    let lowered = tokens
        .iter()
        .skip(start + 1)
        .map(|token| token.to_ascii_lowercase())
        .collect::<Vec<_>>();

    if lowered.is_empty() {
        return Some("biomcp".to_string());
    }

    let mut idx = 0usize;
    while idx < lowered.len() {
        let token = &lowered[idx];
        if is_flag_token(token) {
            normalized.push(token.clone());
            if idx + 1 < lowered.len() && !is_flag_token(&lowered[idx + 1]) {
                normalized.push("<value>".to_string());
                idx += 2;
                continue;
            }
            idx += 1;
            continue;
        }

        if positional_index < 2 {
            normalized.push(token.clone());
        } else if is_section_like_token(token) {
            normalized.push(token.clone());
        } else {
            normalized.push("<arg>".to_string());
        }

        positional_index += 1;
        idx += 1;
    }

    Some(normalized.join(" "))
}

fn is_flag_token(token: &str) -> bool {
    token.starts_with("--") || (token.starts_with('-') && token.len() > 1)
}

fn is_biomcp_binary_token(token: &str) -> bool {
    let basename = token.rsplit('/').next().unwrap_or(token);
    basename == "biomcp" || basename == "biomcp-cli"
}

fn extract_embedded_biomcp(token: &str) -> Option<String> {
    let lower = token.to_ascii_lowercase();
    let idx = lower.find("biomcp")?;
    Some(token[idx..].to_string())
}

fn is_section_like_token(token: &str) -> bool {
    matches!(
        token,
        "all"
            | "pathways"
            | "ontology"
            | "diseases"
            | "protein"
            | "go"
            | "interactions"
            | "civic"
            | "predict"
            | "predictions"
            | "clinvar"
            | "population"
            | "conservation"
            | "cosmic"
            | "cgi"
            | "cbioportal"
            | "gwas"
            | "label"
            | "shortage"
            | "targets"
            | "indications"
            | "entities"
            | "fulltext"
            | "annotations"
            | "eligibility"
            | "locations"
            | "outcomes"
            | "arms"
            | "references"
            | "recommendations"
            | "frequencies"
            | "guidelines"
            | "genes"
            | "events"
            | "enrichment"
            | "domains"
            | "structures"
            | "reactions"
            | "concomitant"
            | "guidance"
            | "trials"
            | "articles"
            | "drugs"
            | "adverse-events"
            | "adverse_event"
    )
}

fn collapse_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_help_command(command: &str) -> bool {
    let lower = command.to_ascii_lowercase();
    lower.contains("biomcp --help")
        || lower.contains("biomcp -h")
        || lower.contains("biomcp help")
        || lower.contains("biomcp list")
}

fn is_skill_read_command(command: &str) -> bool {
    let lower = command.to_ascii_lowercase();
    lower.contains("skills/") || lower.contains("biomcp skill")
}

fn collect_error_messages(value: &Value, out: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, nested) in map {
                let lower_key = key.to_ascii_lowercase();
                if let Value::String(message) = nested
                    && is_error_key(&lower_key)
                {
                    let msg = message.trim();
                    if !msg.is_empty() {
                        out.insert(msg.to_string());
                    }
                }
                collect_error_messages(nested, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_error_messages(item, out);
            }
        }
        _ => {}
    }
}

fn is_error_key(key: &str) -> bool {
    key.contains("error")
        || key.contains("stderr")
        || key.contains("exception")
        || key.contains("failure")
        || key.contains("failed")
}

#[derive(Debug, Clone, Copy)]
enum ErrorCategory {
    Ghost,
    Quoting,
    Api,
    Other,
}

fn classify_error(text: &str) -> ErrorCategory {
    let lower = text.to_ascii_lowercase();
    if lower.contains("ghost") || lower.contains("command not found") || lower.contains("not found")
    {
        return ErrorCategory::Ghost;
    }

    if lower.contains("unterminated")
        || lower.contains("unexpected eof")
        || lower.contains("no closing quotation")
        || lower.contains("quote")
    {
        return ErrorCategory::Quoting;
    }

    if lower.contains("api")
        || lower.contains("http ")
        || lower.contains("timeout")
        || lower.contains("timed out")
        || lower.contains("connection")
        || lower.contains("429")
        || lower.contains("500")
        || lower.contains("502")
        || lower.contains("503")
        || lower.contains("504")
    {
        return ErrorCategory::Api;
    }

    ErrorCategory::Other
}

fn collect_token_usage(value: &Value, usage: &mut SessionTokenUsage) {
    match value {
        Value::Object(map) => {
            for (key, nested) in map {
                let lower_key = key.to_ascii_lowercase();
                if let Some(number) = parse_number(nested) {
                    if is_input_token_key(&lower_key) {
                        usage.input_tokens = usage.input_tokens.saturating_add(number as u64);
                    } else if is_output_token_key(&lower_key) {
                        usage.output_tokens = usage.output_tokens.saturating_add(number as u64);
                    } else if is_cache_read_token_key(&lower_key) {
                        usage.cache_read_tokens =
                            usage.cache_read_tokens.saturating_add(number as u64);
                    } else if is_cache_write_token_key(&lower_key) {
                        usage.cache_write_tokens =
                            usage.cache_write_tokens.saturating_add(number as u64);
                    } else if is_cost_key(&lower_key) {
                        usage.cost_usd += number;
                    }
                }

                collect_token_usage(nested, usage);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_token_usage(item, usage);
            }
        }
        _ => {}
    }
}

fn is_input_token_key(key: &str) -> bool {
    key == "input_tokens" || key == "prompt_tokens"
}

fn is_output_token_key(key: &str) -> bool {
    key == "output_tokens" || key == "completion_tokens"
}

fn is_cache_read_token_key(key: &str) -> bool {
    key == "cache_read_tokens" || key == "cached_tokens"
}

fn is_cache_write_token_key(key: &str) -> bool {
    key == "cache_write_tokens"
}

fn is_cost_key(key: &str) -> bool {
    key == "cost" || key == "cost_usd" || key.ends_with("_cost")
}

fn parse_number(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        _ => None,
    }
}

fn extract_timestamp(value: &Value) -> Option<OffsetDateTime> {
    match value {
        Value::Object(map) => {
            for key in ["timestamp", "time", "created_at", "event_time", "ts"] {
                if let Some(text) = map.get(key).and_then(Value::as_str)
                    && let Ok(ts) = OffsetDateTime::parse(text, &Rfc3339)
                {
                    return Some(ts);
                }
            }

            for nested in map.values() {
                if let Some(ts) = extract_timestamp(nested) {
                    return Some(ts);
                }
            }

            None
        }
        Value::Array(items) => {
            for item in items {
                if let Some(ts) = extract_timestamp(item) {
                    return Some(ts);
                }
            }
            None
        }
        _ => None,
    }
}

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

fn now_rfc3339() -> anyhow::Result<String> {
    Ok(OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .context("failed to format score timestamp")?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(prefix: &str, suffix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "{}-{}-{}{}",
            prefix,
            std::process::id(),
            nanos,
            suffix
        ))
    }

    #[test]
    fn normalize_command_shape_tracks_structure() {
        let shape = normalize_command_shape("biomcp search article -g BRAF --limit 5")
            .expect("shape should parse");
        assert_eq!(shape, "search article -g <value> --limit <value>");

        let shape = normalize_command_shape("bash -lc 'biomcp get gene BRAF pathways'")
            .expect("shape should parse");
        assert_eq!(shape, "get gene <arg> pathways");
    }

    #[test]
    fn score_session_extracts_counts_tokens_errors_and_coverage() {
        let session_path = temp_path("biomcp-session", ".jsonl");
        let expected_path = temp_path("biomcp-expected", ".txt");

        let session = [
            r#"{"timestamp":"2026-02-17T12:00:00Z","tool":{"name":"bash","input":{"cmd":"biomcp get gene BRAF"}},"usage":{"input_tokens":10,"output_tokens":4,"cache_read_tokens":2,"cache_write_tokens":1,"cost_usd":0.001}}"#,
            r#"{"timestamp":"2026-02-17T12:00:02Z","event":{"name":"shell","arguments":{"command":"biomcp --help"}}}"#,
            r#"{"timestamp":"2026-02-17T12:00:04Z","tool":{"name":"bash","input":{"cmd":"cat skills/use-cases/03-trial-searching.md"}}}"#,
            r#"{"timestamp":"2026-02-17T12:00:05Z","stderr":"HTTP 503 from api"}"#,
            r#"{"timestamp":"2026-02-17T12:00:06Z","error":"unterminated quote in command"}"#,
        ]
        .join("\n");

        fs::write(&session_path, session).expect("write session");
        fs::write(
            &expected_path,
            "biomcp get gene TP53\nbiomcp --help\nbiomcp search trial -c melanoma --limit 5\n",
        )
        .expect("write expected");

        let report = score_session_file(&ScoreSessionOptions {
            session: session_path.clone(),
            expected: Some(expected_path.clone()),
            brief: false,
        })
        .expect("score");

        fs::remove_file(&session_path).expect("cleanup session");
        fs::remove_file(&expected_path).expect("cleanup expected");

        assert_eq!(report.total_tool_calls, 3);
        assert_eq!(report.biomcp_commands, 2);
        assert_eq!(report.help_calls, 1);
        assert_eq!(report.skill_reads, 1);
        assert_eq!(report.errors_total, 2);
        assert_eq!(report.error_categories.api, 1);
        assert_eq!(report.error_categories.quoting, 1);
        assert_eq!(report.tokens.input_tokens, 10);
        assert_eq!(report.tokens.output_tokens, 4);
        assert_eq!(report.tokens.cache_read_tokens, 2);
        assert_eq!(report.tokens.cache_write_tokens, 1);
        assert!((report.tokens.cost_usd - 0.001).abs() < 1e-9);
        assert_eq!(report.wall_time_ms, Some(6000));

        let coverage = report.coverage.expect("coverage");
        assert_eq!(coverage.expected_total, 3);
        assert_eq!(coverage.hits, 2);
        assert_eq!(coverage.misses, 1);
        assert_eq!(coverage.extras, 0);
    }

    #[test]
    fn classify_error_detects_expected_categories() {
        assert!(matches!(
            classify_error("ghost command not found"),
            ErrorCategory::Ghost
        ));
        assert!(matches!(
            classify_error("unterminated quote"),
            ErrorCategory::Quoting
        ));
        assert!(matches!(
            classify_error("HTTP 503 from api"),
            ErrorCategory::Api
        ));
        assert!(matches!(classify_error("misc issue"), ErrorCategory::Other));
    }

    #[test]
    fn returns_none_for_non_biomcp_commands() {
        assert!(normalize_command_shape("echo hello").is_none());
    }

    #[test]
    fn fails_on_invalid_jsonl_line() {
        let session_path = temp_path("biomcp-invalid", ".jsonl");
        fs::write(&session_path, "{not-json}\n").expect("write");

        let err = score_session_file(&ScoreSessionOptions {
            session: session_path.clone(),
            expected: None,
            brief: true,
        })
        .expect_err("invalid json should fail");

        fs::remove_file(&session_path).expect("cleanup");

        assert!(err.to_string().contains("invalid JSONL line"));
    }
}
