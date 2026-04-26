//! Cross-entity search-all facade and stable result contracts.

use std::time::Instant;

use futures::future::join_all;
use serde::Serialize;
use serde_json::Value;

use crate::cli::debug_plan::DebugPlan;
use crate::error::BioMcpError;

mod dispatch;
mod format;
mod links;
mod plan;

use dispatch::dispatch_section;
use plan::{PreparedInput, build_dispatch_plan_prepared, build_result_plan};

pub(super) const MAX_SEARCH_ALL_LIMIT: usize = 50;

#[derive(Debug, Clone)]
pub struct SearchAllInput {
    pub gene: Option<String>,
    pub variant: Option<String>,
    pub disease: Option<String>,
    pub drug: Option<String>,
    pub keyword: Option<String>,
    pub since: Option<String>,
    pub limit: usize,
    pub counts_only: bool,
    pub debug_plan: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchAllLink {
    pub rel: String,
    pub title: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchAllSection {
    pub entity: String,
    pub label: String,
    pub count: usize,
    pub total: Option<usize>,
    pub error: Option<String>,
    pub note: Option<String>,
    pub results: Vec<Value>,
    pub links: Vec<SearchAllLink>,
}

impl SearchAllSection {
    pub fn markdown_columns(&self) -> &'static [&'static str] {
        format::markdown_columns(self)
    }

    pub fn markdown_rows(&self) -> Vec<Vec<String>> {
        format::markdown_rows(self)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchAllResults {
    pub query: String,
    pub sections: Vec<SearchAllSection>,
    pub searches_dispatched: usize,
    pub searches_with_results: usize,
    pub wall_time_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) debug_plan: Option<DebugPlan>,
}

#[derive(Debug, Serialize)]
pub(crate) struct SearchAllCountsOnlyJson<'a> {
    pub query: &'a str,
    pub sections: Vec<SearchAllCountsOnlySection<'a>>,
    pub searches_dispatched: usize,
    pub searches_with_results: usize,
    pub wall_time_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_plan: Option<&'a DebugPlan>,
}

#[derive(Debug, Serialize)]
pub(crate) struct SearchAllCountsOnlySection<'a> {
    pub entity: &'a str,
    pub label: &'a str,
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<&'a str>,
}

pub(crate) fn counts_only_json(results: &SearchAllResults) -> SearchAllCountsOnlyJson<'_> {
    SearchAllCountsOnlyJson {
        query: &results.query,
        sections: results
            .sections
            .iter()
            .map(|section| SearchAllCountsOnlySection {
                entity: &section.entity,
                label: &section.label,
                count: section.total.unwrap_or(section.count),
                note: section.note.as_deref(),
                error: section.error.as_deref(),
            })
            .collect(),
        searches_dispatched: results.searches_dispatched,
        searches_with_results: results.searches_with_results,
        wall_time_ms: results.wall_time_ms,
        debug_plan: results.debug_plan.as_ref(),
    }
}

#[derive(Debug, Clone)]
pub struct DispatchSpec {
    pub entity: &'static str,
    pub(super) kind: SectionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum SectionKind {
    Gene,
    Variant,
    Disease,
    Drug,
    Trial,
    Article,
    Pathway,
    Pgx,
    Gwas,
    AdverseEvent,
}

impl SectionKind {
    pub(super) fn from_entity(entity: &str) -> Option<Self> {
        match entity {
            "gene" => Some(Self::Gene),
            "variant" => Some(Self::Variant),
            "disease" => Some(Self::Disease),
            "drug" => Some(Self::Drug),
            "trial" => Some(Self::Trial),
            "article" => Some(Self::Article),
            "pathway" => Some(Self::Pathway),
            "pgx" => Some(Self::Pgx),
            "gwas" => Some(Self::Gwas),
            "adverse-event" => Some(Self::AdverseEvent),
            _ => None,
        }
    }

    pub(super) fn entity(self) -> &'static str {
        match self {
            Self::Gene => "gene",
            Self::Variant => "variant",
            Self::Disease => "disease",
            Self::Drug => "drug",
            Self::Trial => "trial",
            Self::Article => "article",
            Self::Pathway => "pathway",
            Self::Pgx => "pgx",
            Self::Gwas => "gwas",
            Self::AdverseEvent => "adverse-event",
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Gene => "Genes",
            Self::Variant => "Variants",
            Self::Disease => "Diseases",
            Self::Drug => "Drugs",
            Self::Trial => "Trials",
            Self::Article => "Articles",
            Self::Pathway => "Pathways",
            Self::Pgx => "PGx",
            Self::Gwas => "GWAS",
            Self::AdverseEvent => "Adverse Events",
        }
    }
}

pub fn build_dispatch_plan(input: &SearchAllInput) -> Vec<DispatchSpec> {
    let Ok(prepared) = PreparedInput::new(input) else {
        return Vec::new();
    };
    build_dispatch_plan_prepared(&prepared)
}

pub async fn dispatch(input: &SearchAllInput) -> Result<SearchAllResults, BioMcpError> {
    let prepared = PreparedInput::new(input)?;
    let plan = build_dispatch_plan_prepared(&prepared);
    let started = Instant::now();

    let sections = join_all(
        plan.iter()
            .map(|spec| dispatch_section(spec.kind, &prepared)),
    )
    .await;

    let searches_dispatched = sections.len();
    let searches_with_results = sections
        .iter()
        .filter(|section| section.error.is_none() && section.count > 0)
        .count();

    Ok(SearchAllResults {
        query: prepared.query_summary(),
        debug_plan: prepared
            .debug_plan
            .then(|| build_result_plan(&prepared, &sections)),
        sections,
        searches_dispatched,
        searches_with_results,
        wall_time_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
    })
}

#[cfg(test)]
mod tests {
    mod dispatch;
    mod format;
    mod links;
    mod plan;
}
