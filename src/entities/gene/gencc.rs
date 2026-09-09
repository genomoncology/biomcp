//! Gene-card projection and resolved-identity matching for GenCC.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::GENE_SECTION_GENCC;
use crate::entities::gene::Gene;
use crate::entities::section_outcome::SectionOutcome;
use crate::sources::gencc::{
    GenCcAssertion, GenCcClient, GenCcData, GenCcFreshness, GenCcOperation, GenCcResult,
    GenCcStatus,
};

const IDENTITY_MESSAGE: &str =
    "GenCC gene identity is inconclusive; no GenCC absence can be concluded.";
const STALE_FAILED_MESSAGE: &str =
    "GenCC refresh failed; results come from the last validated dataset.";
const STALE_PROGRESS_MESSAGE: &str =
    "GenCC refresh is still in progress; results come from the last validated dataset.";
const UNAVAILABLE_MESSAGE: &str = "GenCC data is unavailable; no GenCC absence can be concluded.";
const UNAVAILABLE_PROGRESS_MESSAGE: &str =
    "GenCC refresh is still in progress; no GenCC absence can be concluded.";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneGenCc {
    pub assertions: Vec<GenCcAssertion>,
    pub total_matching_assertions: usize,
    pub truncated: bool,
    pub status: GenCcStatus,
}

impl GeneGenCc {
    fn identity() -> Self {
        Self {
            assertions: Vec::new(),
            total_matching_assertions: 0,
            truncated: false,
            status: GenCcStatus {
                freshness: GenCcFreshness::Unavailable,
                result: GenCcResult::Unknown,
                operation: GenCcOperation::IdentityMatch,
                checked_at: None,
                retrieved_at: None,
                attempted_at: None,
                etag: None,
                last_modified: None,
                upstream_version: None,
                message: Some(IDENTITY_MESSAGE.to_string()),
            },
        }
    }
}

pub(super) async fn add_section(gene: &mut Gene, hgnc: Result<Vec<String>, ()>, timeout: Duration) {
    let (section, outcome) = fetch_section(&gene.symbol, hgnc, timeout).await;
    gene.gencc = Some(section);
    gene.section_outcomes.complete(GENE_SECTION_GENCC, outcome);
}

pub(super) async fn fetch_section(
    symbol: &str,
    hgnc: Result<Vec<String>, ()>,
    timeout: Duration,
) -> (GeneGenCc, SectionOutcome) {
    let deadline = tokio::time::Instant::now() + timeout;
    let hgnc = match hgnc {
        Ok(values) if values.len() <= 1 && valid_symbol(symbol) => values.into_iter().next(),
        _ => {
            let section = GeneGenCc::identity();
            let outcome = section_outcome(&section);
            return (section, outcome);
        }
    };
    let client = match GenCcClient::new() {
        Ok(client) => client,
        Err(()) => {
            let mut section = GeneGenCc::identity();
            section.status.operation = GenCcOperation::InitialDownload;
            section.status.message =
                Some("GenCC data is unavailable; no GenCC absence can be concluded.".into());
            return (
                section,
                SectionOutcome::unavailable(
                    "GenCC data is unavailable; no GenCC absence can be concluded.",
                ),
            );
        }
    };
    let reserve = timeout.div_f32(4.0).min(Duration::from_millis(500));
    let acquisition_deadline = deadline.checked_sub(reserve).unwrap_or(deadline);
    let data = client.acquire_until(acquisition_deadline, deadline).await;
    project_until(symbol, hgnc.as_deref(), data, deadline).await
}

#[cfg(test)]
fn project(
    symbol: &str,
    resolved_hgnc: Option<&str>,
    data: GenCcData,
) -> (GeneGenCc, SectionOutcome) {
    project_cancellable(symbol, resolved_hgnc, data, &AtomicBool::new(false))
        .expect("uncancelled GenCC projection")
}

async fn project_until(
    symbol: &str,
    resolved_hgnc: Option<&str>,
    data: GenCcData,
    deadline: tokio::time::Instant,
) -> (GeneGenCc, SectionOutcome) {
    let status = data.status.clone();
    if tokio::time::Instant::now() >= deadline {
        return projection_unavailable(status);
    }
    let symbol = symbol.to_string();
    let resolved_hgnc = resolved_hgnc.map(str::to_string);
    let cancelled = Arc::new(AtomicBool::new(false));
    let worker_cancelled = Arc::clone(&cancelled);
    let mut worker = tokio::task::spawn_blocking(move || {
        project_cancellable(&symbol, resolved_hgnc.as_deref(), data, &worker_cancelled)
    });
    match tokio::time::timeout_at(deadline, &mut worker).await {
        Ok(Ok(Some(result))) => result,
        Ok(Ok(None) | Err(_)) => projection_unavailable(status),
        Err(_) => {
            cancelled.store(true, Ordering::Relaxed);
            let _ = worker.await;
            projection_unavailable(status)
        }
    }
}

fn project_cancellable(
    symbol: &str,
    resolved_hgnc: Option<&str>,
    data: GenCcData,
    cancelled: &AtomicBool,
) -> Option<(GeneGenCc, SectionOutcome)> {
    // Keep the shared generation lock alive through the complete index projection.
    let _lease = data.lease;
    let Some(dataset) = data.dataset else {
        let section = GeneGenCc {
            assertions: Vec::new(),
            total_matching_assertions: 0,
            truncated: false,
            status: data.status,
        };
        let outcome = section_outcome(&section);
        return Some((section, outcome));
    };
    let mut symbol_ids = Vec::new();
    for row in dataset.assertions() {
        if cancelled.load(Ordering::Relaxed) {
            return None;
        }
        if row.gene.label.eq_ignore_ascii_case(symbol) && !symbol_ids.contains(&row.gene.id) {
            symbol_ids.push(row.gene.id.clone());
        }
    }
    let selected_hgnc = if let Some(hgnc) = resolved_hgnc {
        let mut curie_symbols = Vec::new();
        for row in dataset.assertions() {
            if cancelled.load(Ordering::Relaxed) {
                return None;
            }
            if row.gene.id == hgnc {
                curie_symbols.push(row.gene.label.as_str());
            }
        }
        if symbol_ids.iter().any(|id| id != hgnc)
            || curie_symbols
                .iter()
                .any(|row_symbol| !row_symbol.eq_ignore_ascii_case(symbol))
        {
            let section = identity_after_lifecycle(data.status);
            let outcome = section_outcome(&section);
            return Some((section, outcome));
        }
        hgnc.to_string()
    } else {
        match symbol_ids.as_slice() {
            [] => {
                let mut section = GeneGenCc {
                    assertions: Vec::new(),
                    total_matching_assertions: 0,
                    truncated: false,
                    status: data.status,
                };
                section.status.result = GenCcResult::Empty;
                let outcome = section_outcome(&section);
                return Some((section, outcome));
            }
            [only] => only.clone(),
            _ => {
                let section = identity_after_lifecycle(data.status);
                let outcome = section_outcome(&section);
                return Some((section, outcome));
            }
        }
    };
    let mut assertions = Vec::new();
    for row in dataset.assertions() {
        if cancelled.load(Ordering::Relaxed) {
            return None;
        }
        if row.gene.label.eq_ignore_ascii_case(symbol) && row.gene.id == selected_hgnc {
            assertions.push(row.clone());
        }
    }
    let total_matching_assertions = assertions.len();
    assertions.truncate(100);
    let mut section = GeneGenCc {
        assertions,
        total_matching_assertions,
        truncated: total_matching_assertions > 100,
        status: data.status,
    };
    section.status.result = if total_matching_assertions == 0 {
        GenCcResult::Empty
    } else {
        GenCcResult::Data
    };
    let outcome = section_outcome(&section);
    Some((section, outcome))
}

fn projection_unavailable(mut status: GenCcStatus) -> (GeneGenCc, SectionOutcome) {
    status.freshness = GenCcFreshness::Unavailable;
    status.result = GenCcResult::Unknown;
    status.message = Some(UNAVAILABLE_MESSAGE.into());
    let section = GeneGenCc {
        assertions: Vec::new(),
        total_matching_assertions: 0,
        truncated: false,
        status,
    };
    let outcome = section_outcome(&section);
    (section, outcome)
}

fn identity_after_lifecycle(mut status: GenCcStatus) -> GeneGenCc {
    status.freshness = GenCcFreshness::Unavailable;
    status.result = GenCcResult::Unknown;
    status.operation = GenCcOperation::IdentityMatch;
    status.message = Some(IDENTITY_MESSAGE.into());
    GeneGenCc {
        assertions: Vec::new(),
        total_matching_assertions: 0,
        truncated: false,
        status,
    }
}

fn valid_symbol(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

pub(super) fn section_outcome(section: &GeneGenCc) -> SectionOutcome {
    let message = match (
        section.status.operation,
        section.status.freshness,
        section.status.message.as_deref(),
    ) {
        (GenCcOperation::IdentityMatch, _, _) => IDENTITY_MESSAGE,
        (GenCcOperation::RefreshDeferred, GenCcFreshness::Stale, _) => STALE_PROGRESS_MESSAGE,
        (GenCcOperation::RefreshDeferred, _, _) => UNAVAILABLE_PROGRESS_MESSAGE,
        (_, GenCcFreshness::Stale, _) => STALE_FAILED_MESSAGE,
        _ => UNAVAILABLE_MESSAGE,
    };
    match (section.status.freshness, section.status.result) {
        (GenCcFreshness::Fresh, GenCcResult::Data) => SectionOutcome::data("GenCC"),
        (GenCcFreshness::Fresh, GenCcResult::Empty) => SectionOutcome::empty("GenCC"),
        (GenCcFreshness::Stale, GenCcResult::Data) => SectionOutcome::degraded(["GenCC"], message),
        _ => SectionOutcome::unavailable(message),
    }
}

#[cfg(test)]
mod tests;
