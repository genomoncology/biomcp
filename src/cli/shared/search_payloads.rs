//! Pagination metadata and the search JSON payload shapes shared by every entity.

#[derive(Debug, Clone, serde::Serialize)]
pub(in crate::cli) struct PaginationMeta {
    pub offset: usize,
    pub limit: usize,
    pub returned: usize,
    pub total: Option<usize>,
    pub has_more: bool,
    pub next_page_token: Option<String>,
}

impl PaginationMeta {
    pub(in crate::cli) fn offset(
        offset: usize,
        limit: usize,
        returned: usize,
        total: Option<usize>,
    ) -> Self {
        let has_more = total
            .map(|value| offset.saturating_add(returned) < value)
            .unwrap_or(returned == limit);
        Self {
            offset,
            limit,
            returned,
            total,
            has_more,
            next_page_token: None,
        }
    }
}

// dead-code reason: shared::SearchJsonResponse is exercised by binary dispatch or CLI contracts
#[cfg_attr(not(test), allow(dead_code))]
#[derive(serde::Serialize)]
struct SearchJsonResponse<T: serde::Serialize> {
    pagination: PaginationMeta,
    count: usize,
    results: Vec<T>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(in crate::cli) struct SearchJsonMeta {
    pub(in crate::cli) next_commands: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::cli) suggestions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::cli) workflow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::cli) workflow_rationale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::cli) workflow_playbook: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(in crate::cli) section_sources: Vec<crate::render::provenance::SectionSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::cli) upstream_total: Option<usize>,
    /// Notes the search wants to carry, such as a filled provider window.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(in crate::cli) notes: Vec<String>,
}

impl SearchJsonMeta {
    pub(in crate::cli) fn with_section_sources(
        mut self,
        section_sources: Vec<crate::render::provenance::SectionSource>,
    ) -> Self {
        self.section_sources = section_sources;
        self
    }
}

#[derive(serde::Serialize)]
pub(in crate::cli) struct SearchJsonResponseWithMeta<T: serde::Serialize> {
    pub(in crate::cli) pagination: PaginationMeta,
    pub(in crate::cli) count: usize,
    pub(in crate::cli) results: Vec<T>,
    /// The source release the rows came from, when the entity reports one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::cli) data_as_of: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::cli) data_as_of_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::cli) _meta: Option<SearchJsonMeta>,
}

// dead-code reason: shared::search_json is exercised by binary dispatch or CLI contracts
#[cfg_attr(not(test), allow(dead_code))]
pub(in crate::cli) fn search_json<T: serde::Serialize>(
    results: Vec<T>,
    pagination: PaginationMeta,
) -> anyhow::Result<String> {
    let count = results.len();
    crate::render::json::to_pretty(&SearchJsonResponse {
        pagination,
        count,
        results,
    })
    .map_err(Into::into)
}

pub(in crate::cli) fn normalize_next_commands(next_commands: Vec<String>) -> Vec<String> {
    next_commands
        .into_iter()
        .map(|command| command.trim().to_string())
        .filter(|command| !command.is_empty())
        .collect()
}

pub(in crate::cli) fn search_meta(next_commands: Vec<String>) -> Option<SearchJsonMeta> {
    search_meta_with_suggestions(next_commands, None)
}

pub(in crate::cli) fn search_meta_with_section_sources(
    next_commands: Vec<String>,
    section_sources: Vec<crate::render::provenance::SectionSource>,
) -> Option<SearchJsonMeta> {
    let meta = search_meta(next_commands).unwrap_or(SearchJsonMeta {
        next_commands: Vec::new(),
        suggestions: None,
        workflow: None,
        workflow_rationale: None,
        workflow_playbook: None,
        section_sources: Vec::new(),
        upstream_total: None,
        notes: Vec::new(),
    });
    (!meta.next_commands.is_empty() || !section_sources.is_empty())
        .then(|| meta.with_section_sources(section_sources))
}

pub(in crate::cli) fn search_meta_with_suggestions(
    next_commands: Vec<String>,
    suggestions: Option<Vec<String>>,
) -> Option<SearchJsonMeta> {
    search_meta_with_workflow(next_commands, suggestions, None)
}

pub(in crate::cli) fn search_meta_with_workflow(
    next_commands: Vec<String>,
    suggestions: Option<Vec<String>>,
    workflow: Option<crate::workflow_ladders::WorkflowMeta>,
) -> Option<SearchJsonMeta> {
    let next_commands = normalize_next_commands(next_commands);
    let suggestions = suggestions.map(normalize_next_commands);
    let (workflow, workflow_rationale, workflow_playbook) = workflow
        .map(|meta| {
            (
                Some(meta.workflow),
                Some(meta.rationale),
                Some(meta.playbook),
            )
        })
        .unwrap_or((None, None, None));
    (!next_commands.is_empty() || suggestions.is_some() || workflow.is_some()).then_some(
        SearchJsonMeta {
            next_commands,
            suggestions,
            workflow,
            workflow_rationale,
            workflow_playbook,
            section_sources: Vec::new(),
            upstream_total: None,
            notes: Vec::new(),
        },
    )
}

pub(in crate::cli) fn search_json_with_meta<T: serde::Serialize>(
    results: Vec<T>,
    pagination: PaginationMeta,
    next_commands: Vec<String>,
) -> anyhow::Result<String> {
    search_json_with_meta_and_suggestions(results, pagination, next_commands, None)
}

pub(in crate::cli) fn search_json_with_meta_and_suggestions<T: serde::Serialize>(
    results: Vec<T>,
    pagination: PaginationMeta,
    next_commands: Vec<String>,
    suggestions: Option<Vec<String>>,
) -> anyhow::Result<String> {
    let count = results.len();
    crate::render::json::to_pretty(&SearchJsonResponseWithMeta {
        pagination,
        count,
        results,
        data_as_of: None,
        data_as_of_kind: None,
        _meta: search_meta_with_suggestions(next_commands, suggestions),
    })
    .map_err(Into::into)
}

/// Search JSON that carries the source release and any notes. Only entities
/// that report a release use it, so no other entity's output changes.
pub(in crate::cli) fn search_json_with_data_as_of<T: serde::Serialize>(
    results: Vec<T>,
    pagination: PaginationMeta,
    next_commands: Vec<String>,
    notes: Vec<String>,
    data_as_of: String,
    data_as_of_kind: String,
) -> anyhow::Result<String> {
    let count = results.len();
    let mut meta = search_meta(next_commands).unwrap_or(SearchJsonMeta {
        next_commands: Vec::new(),
        suggestions: None,
        workflow: None,
        workflow_rationale: None,
        workflow_playbook: None,
        section_sources: Vec::new(),
        upstream_total: None,
        notes: Vec::new(),
    });
    meta.notes = notes;
    crate::render::json::to_pretty(&SearchJsonResponseWithMeta {
        pagination,
        count,
        results,
        data_as_of: Some(data_as_of),
        data_as_of_kind: Some(data_as_of_kind),
        _meta: Some(meta),
    })
    .map_err(Into::into)
}

pub(in crate::cli) fn pagination_footer_offset(meta: &PaginationMeta) -> String {
    crate::render::markdown::pagination_footer(meta.offset, meta.limit, meta.returned, meta.total)
}
