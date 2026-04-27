//! Session command normalization and expected-command coverage.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::Context;

use super::super::types::SessionCoverage;

pub(super) fn compute_coverage(
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

pub(super) fn normalize_command_shape(command: &str) -> Option<String> {
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
            | "expression"
            | "hpa"
            | "druggability"
            | "clingen"
            | "constraint"
            | "disgenet"
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

#[cfg(test)]
#[path = "tests/normalize.rs"]
mod tests;
