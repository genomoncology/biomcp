//! Gene-card projection and resolved-identity matching for GenCC.

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

async fn fetch_section(
    symbol: &str,
    hgnc: Result<Vec<String>, ()>,
    timeout: Duration,
) -> (GeneGenCc, SectionOutcome) {
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
    let data = match tokio::time::timeout(timeout, client.acquire()).await {
        Ok(data) => data,
        Err(_) => GenCcData {
            dataset: None,
            status: GenCcStatus {
                freshness: GenCcFreshness::Unavailable,
                result: GenCcResult::Unknown,
                operation: GenCcOperation::RefreshDeferred,
                checked_at: None,
                retrieved_at: None,
                attempted_at: None,
                etag: None,
                last_modified: None,
                upstream_version: None,
                message: Some(
                    "GenCC refresh is still in progress; no GenCC absence can be concluded.".into(),
                ),
            },
        },
    };
    project(symbol, hgnc.as_deref(), data)
}

fn project(
    symbol: &str,
    resolved_hgnc: Option<&str>,
    data: GenCcData,
) -> (GeneGenCc, SectionOutcome) {
    let Some(dataset) = data.dataset else {
        let section = GeneGenCc {
            assertions: Vec::new(),
            total_matching_assertions: 0,
            truncated: false,
            status: data.status,
        };
        let outcome = section_outcome(&section);
        return (section, outcome);
    };
    let symbol_ids = dataset.symbol_hgnc_ids(symbol);
    let selected_hgnc = if let Some(hgnc) = resolved_hgnc {
        let curie_symbols = dataset
            .assertions()
            .iter()
            .filter(|row| row.gene.id == hgnc)
            .map(|row| row.gene.label.as_str())
            .collect::<Vec<_>>();
        if symbol_ids.iter().any(|id| id != hgnc)
            || curie_symbols
                .iter()
                .any(|row_symbol| !row_symbol.eq_ignore_ascii_case(symbol))
        {
            let section = identity_after_lifecycle(data.status);
            let outcome = section_outcome(&section);
            return (section, outcome);
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
                return (section, outcome);
            }
            [only] => only.clone(),
            _ => {
                let section = identity_after_lifecycle(data.status);
                let outcome = section_outcome(&section);
                return (section, outcome);
            }
        }
    };
    let mut assertions = dataset.matching(symbol, &selected_hgnc);
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
    match (section.status.freshness, section.status.result) {
        (GenCcFreshness::Fresh, GenCcResult::Data) => SectionOutcome::data("GenCC"),
        (GenCcFreshness::Fresh, GenCcResult::Empty) => SectionOutcome::empty("GenCC"),
        (GenCcFreshness::Stale, GenCcResult::Data) => SectionOutcome::degraded(
            ["GenCC"],
            "GenCC refresh failed; results come from the last validated dataset.",
        ),
        _ => SectionOutcome::unavailable(match section.status.message.as_deref() {
            Some(IDENTITY_MESSAGE) => IDENTITY_MESSAGE,
            Some("GenCC refresh is still in progress; no GenCC absence can be concluded.") => {
                "GenCC refresh is still in progress; no GenCC absence can be concluded."
            }
            _ => "GenCC data is unavailable; no GenCC absence can be concluded.",
        }),
    }
}

#[cfg(test)]
mod tests;
