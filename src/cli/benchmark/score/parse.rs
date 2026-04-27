//! Session JSONL parsing, tool-call extraction, errors, tokens, and timestamps.

use std::collections::BTreeSet;

use serde_json::{Map, Value};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use super::super::types::SessionTokenUsage;

#[derive(Debug)]
pub(super) struct ToolCall {
    pub(super) tool_name: String,
    pub(super) command: String,
}

pub(super) fn collect_tool_calls(value: &Value, out: &mut Vec<ToolCall>) {
    match value {
        Value::Object(map) => {
            if let Some(name) = extract_tool_name(map)
                && let Some(command) = extract_command_from_object(map)
                && is_biomcp_tool_name(&name)
            {
                out.push(ToolCall {
                    tool_name: name,
                    command,
                });
            }

            if let Some(function) = map.get("function").and_then(Value::as_object)
                && let Some(name) = extract_tool_name(function)
                && let Some(command) = extract_command_from_object(function)
                && is_biomcp_tool_name(&name)
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

pub(super) fn is_biomcp_tool_name(name: &str) -> bool {
    let normalized = name.trim().to_ascii_lowercase();
    normalized == "biomcp"
        || normalized == "bash"
        || normalized == "shell"
        || normalized.ends_with(".biomcp")
        || normalized.ends_with(".bash")
        || normalized.ends_with(".shell")
}

pub(super) fn is_biomcp_shell_tool(name: &str) -> bool {
    is_biomcp_tool_name(name)
}

pub(super) fn collapse_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(super) fn is_help_command(command: &str) -> bool {
    let lower = command.to_ascii_lowercase();
    lower.contains("biomcp --help")
        || lower.contains("biomcp -h")
        || lower.contains("biomcp help")
        || lower.contains("biomcp list")
}

pub(super) fn is_skill_read_command(command: &str) -> bool {
    let lower = command.to_ascii_lowercase();
    lower.contains("skills/") || lower.contains("biomcp skill")
}

pub(super) fn collect_error_messages(value: &Value, out: &mut BTreeSet<String>) {
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
pub(super) enum ErrorCategory {
    Ghost,
    Quoting,
    Api,
    Other,
}

pub(super) fn classify_error(text: &str) -> ErrorCategory {
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

pub(super) fn collect_token_usage(value: &Value, usage: &mut SessionTokenUsage) {
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

pub(super) fn extract_timestamp(value: &Value) -> Option<OffsetDateTime> {
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
