//! Entity extraction, cleanup, normalization, and command-anchor quoting for suggestions.

use regex::Regex;

use crate::render::markdown::shell_quote_arg;

use super::QuestionContext;
use super::patterns::*;

pub(super) fn extract_variant_identifier(ctx: &QuestionContext<'_>) -> Option<String> {
    rsid_re()
        .find(ctx.original)
        .map(|m| m.as_str().to_ascii_lowercase())
        .or_else(|| {
            gene_variant_re()
                .find(ctx.original)
                .and_then(|m| clean_anchor(m.as_str()))
        })
        .or_else(|| {
            hgvs_re()
                .find(ctx.original)
                .and_then(|m| clean_anchor(m.as_str()))
        })
}

pub(super) fn extract_article_identifier(ctx: &QuestionContext<'_>) -> Option<String> {
    pmid_re()
        .captures(ctx.original)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        .or_else(|| {
            pmcid_re()
                .captures(ctx.original)
                .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        })
        .or_else(|| doi_re().find(ctx.original).map(|m| m.as_str().to_string()))
        .or_else(|| {
            bare_article_id_re()
                .find(ctx.original)
                .map(|m| m.as_str().to_string())
        })
}

pub(super) fn extract_gene_symbol(ctx: &QuestionContext<'_>) -> Option<String> {
    if let Some(symbol) = capture_clean(explicit_gene_re(), ctx, 1) {
        let symbol = symbol.to_ascii_uppercase();
        if !is_gene_stopword(&symbol) {
            return Some(symbol);
        }
    }

    for matched in gene_symbol_re().find_iter(ctx.original) {
        let symbol = matched.as_str();
        if is_gene_stopword(symbol) {
            continue;
        }
        return Some(symbol.to_string());
    }
    None
}

pub(super) fn detect_regulatory_region(ctx: &QuestionContext<'_>) -> Option<&'static str> {
    if ctx.has_any(&["fda", "us", "u s", "united states"]) {
        return Some("us");
    }
    if ctx.has_any(&["ema", "eu", "europe", "european"]) {
        return Some("eu");
    }
    if ctx.has_any(&["who", "world health organization"]) {
        return Some("who");
    }
    None
}

pub(super) fn trial_intervention_anchor(ctx: &QuestionContext<'_>) -> Option<String> {
    capture_clean_any(trial_intervention_re(), ctx, &[1, 2]).or_else(|| {
        if ctx.has_any(&["with"]) {
            capture_clean(trial_with_re(), ctx, 1)
        } else {
            None
        }
    })
}

pub(super) fn content_anchor_before_terms(
    ctx: &QuestionContext<'_>,
    terms: &[&str],
) -> Option<String> {
    let lower = ctx.original.to_ascii_lowercase();
    for term in terms {
        if let Some(index) = lower.find(term) {
            let before = &ctx.original[..index];
            if let Some(anchor) = clean_anchor(before) {
                return Some(anchor);
            }
        }
    }
    None
}

pub(super) fn syndrome_pair(ctx: &QuestionContext<'_>) -> Option<(String, String)> {
    [
        syndrome_compare_re(),
        syndrome_difference_re(),
        syndrome_vs_re(),
        syndrome_confused_re(),
    ]
    .iter()
    .find_map(|regex| capture_pair(regex, ctx))
}

pub(super) fn negative_terms(ctx: &QuestionContext<'_>) -> Option<(String, String)> {
    [linked_terms_re(), cause_terms_re(), evidence_terms_re()]
        .iter()
        .find_map(|regex| capture_pair(regex, ctx))
}

pub(super) fn mechanism_drug_anchor(ctx: &QuestionContext<'_>) -> Option<String> {
    capture_clean_any(mechanism_resistance_re(), ctx, &[1, 2, 3])
        .or_else(|| capture_clean(mechanism_work_re(), ctx, 1))
        .or_else(|| capture_clean(mechanism_of_re(), ctx, 1))
        .or_else(|| capture_clean(mechanism_topic_re(), ctx, 1))
}

pub(super) fn mechanism_gene_topic(ctx: &QuestionContext<'_>, gene: &str) -> String {
    cleanup_question_topic(ctx.original)
        .map(|topic| topic.replace(gene, "").trim().to_string())
        .and_then(|topic| clean_anchor(&topic))
        .unwrap_or_else(|| "pathway mechanism".to_string())
}

pub(super) fn cleanup_question_topic(value: &str) -> Option<String> {
    let mut topic = clean_anchor(value)?;
    for prefix in [
        "what pathway explains ",
        "what mechanism explains ",
        "what is the mechanism of ",
        "what is the pathway for ",
        "is there evidence for ",
        "is ",
        "are ",
        "does ",
        "do ",
    ] {
        if topic.to_ascii_lowercase().starts_with(prefix) {
            topic = topic[prefix.len()..].trim().to_string();
        }
    }
    clean_anchor(&topic)
}

pub(super) fn capture_clean(
    regex: &'static Regex,
    ctx: &QuestionContext<'_>,
    index: usize,
) -> Option<String> {
    let captures = regex.captures(ctx.original)?;
    clean_anchor(captures.get(index)?.as_str())
}

pub(super) fn capture_clean_any(
    regex: &'static Regex,
    ctx: &QuestionContext<'_>,
    indexes: &[usize],
) -> Option<String> {
    let captures = regex.captures(ctx.original)?;
    indexes
        .iter()
        .filter_map(|index| captures.get(*index))
        .find_map(|matched| clean_anchor(matched.as_str()))
}

pub(super) fn capture_pair(
    regex: &'static Regex,
    ctx: &QuestionContext<'_>,
) -> Option<(String, String)> {
    let captures = regex.captures(ctx.original)?;
    let first = clean_anchor(captures.get(1)?.as_str())?;
    let second = clean_anchor(captures.get(2)?.as_str())?;
    Some((first, second))
}

pub(super) fn clean_anchor(raw: &str) -> Option<String> {
    let collapsed = raw
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_matches(|c: char| {
            matches!(
                c,
                '?' | '.' | ',' | ';' | ':' | '!' | '"' | '\'' | '(' | ')' | '[' | ']'
            )
        })
        .trim()
        .to_string();
    if collapsed.is_empty() {
        return None;
    }

    let mut value = collapsed;
    loop {
        let lower = value.to_ascii_lowercase();
        let Some(prefix) = ANCHOR_PREFIXES
            .iter()
            .find(|prefix| lower.starts_with(**prefix))
        else {
            break;
        };
        value = value[prefix.len()..].trim().to_string();
        if value.is_empty() {
            return None;
        }
    }

    let normalized = normalize_text(&value);
    if normalized.len() < 2 || STOP_ANCHORS.contains(&normalized.as_str()) {
        return None;
    }
    if normalized
        .split_whitespace()
        .all(|word| STOP_ANCHORS.contains(&word))
    {
        return None;
    }
    Some(value)
}

pub(super) fn quote(value: &str) -> String {
    shell_quote_arg(value)
}

pub(super) fn normalize_text(value: &str) -> String {
    let mut out = String::new();
    let mut previous_space = true;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            previous_space = false;
        } else if !previous_space {
            out.push(' ');
            previous_space = true;
        }
    }
    out.trim().to_string()
}

pub(super) fn contains_phrase(normalized: &str, phrase: &str) -> bool {
    let phrase = normalize_text(phrase);
    if phrase.is_empty() {
        return false;
    }
    let haystack = format!(" {normalized} ");
    let needle = format!(" {phrase} ");
    haystack.contains(&needle)
}

pub(super) fn is_gene_stopword(symbol: &str) -> bool {
    let upper = symbol.to_ascii_uppercase();
    let lower = symbol.to_ascii_lowercase();
    GENE_STOPWORDS.contains(&upper.as_str()) || STOP_ANCHORS.contains(&lower.as_str())
}
