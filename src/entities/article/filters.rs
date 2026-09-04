//! Article filter normalization and result-level filter helpers.

use crate::error::BioMcpError;
use crate::utils::date::validate_since;

use super::{ArticleSearchFilters, ArticleSearchResult};

pub(super) fn is_preprint_journal(journal: &str) -> bool {
    let j = journal.to_ascii_lowercase();
    j.contains("biorxiv") || j.contains("medrxiv") || j.contains("arxiv")
}

pub(super) fn normalize_article_type(value: &str) -> Result<&'static str, BioMcpError> {
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "review" => Ok("review"),
        "research" | "research-article" => Ok("research-article"),
        "case-reports" => Ok("case-reports"),
        "meta-analysis" | "metaanalysis" => Ok("meta-analysis"),
        _ => Err(BioMcpError::InvalidArgument(
            "--type must be one of: review, research, research-article, case-reports, meta-analysis".into(),
        )),
    }
}

fn relabel_date_argument_error(err: BioMcpError, flag_name: &str) -> BioMcpError {
    if let BioMcpError::InvalidArgument(message) = err {
        BioMcpError::InvalidArgument(message.replace("--since", flag_name))
    } else {
        err
    }
}

fn normalized_date_bound(
    value: Option<&str>,
    flag_name: &str,
) -> Result<Option<String>, BioMcpError> {
    value
        .map(|value| {
            validate_since(value).map_err(|err| relabel_date_argument_error(err, flag_name))
        })
        .transpose()
}

#[derive(Clone, Copy)]
enum NativeKeywordField {
    Author,
    Journal,
    Affiliation,
}

#[derive(Clone, Copy)]
enum ReservedKeywordField {
    Gene,
    Disease,
    Drug,
}

fn reserved_keyword_field(keyword: &str) -> Option<ReservedKeywordField> {
    const FIELDS: [(&str, ReservedKeywordField); 3] = [
        ("gene:", ReservedKeywordField::Gene),
        ("disease:", ReservedKeywordField::Disease),
        ("drug:", ReservedKeywordField::Drug),
    ];
    let keyword = keyword.trim();

    for (label, field) in FIELDS {
        for (start, _) in keyword.char_indices() {
            let end = start + label.len();
            if keyword
                .get(start..end)
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case(label))
                && (start == 0
                    || keyword[..start]
                        .chars()
                        .next_back()
                        .is_some_and(|previous| previous.is_whitespace() || previous == '('))
            {
                return Some(field);
            }
        }
    }
    None
}

pub(crate) fn validate_query_inputs(
    gene: Option<&str>,
    keyword: Option<&str>,
) -> Result<(), BioMcpError> {
    if let Some(gene) = gene
        && (gene.trim().is_empty() || gene.trim().chars().any(char::is_whitespace))
    {
        return Err(BioMcpError::InvalidArgument(
            "gene accepts one symbol, for example TPMT. Put additional concepts in keyword: use --gene TPMT --keyword mercaptopurine for CLI or raw MCP, or typed MCP fields \"gene\":\"TPMT\" and \"keyword\":[\"mercaptopurine\"]."
                .into(),
        ));
    }

    if let Some(field) = keyword.and_then(reserved_keyword_field) {
        let (label, flag, example) = match field {
            ReservedKeywordField::Gene => ("gene:", "--gene RB1", r#""gene":"RB1""#),
            ReservedKeywordField::Disease => {
                ("disease:", "--disease melanoma", r#""disease":"melanoma""#)
            }
            ReservedKeywordField::Drug => {
                ("drug:", "--drug vemurafenib", r#""drug":"vemurafenib""#)
            }
        };
        return Err(BioMcpError::InvalidArgument(format!(
            "keyword is provider-neutral and does not accept {label} filter syntax. Use {flag} for CLI or raw MCP, or the typed MCP field, for example {example}."
        )));
    }
    Ok(())
}

fn native_keyword_field(keyword: &str) -> Option<NativeKeywordField> {
    const SUFFIX_FIELDS: [(&str, NativeKeywordField); 5] = [
        ("[author]", NativeKeywordField::Author),
        ("[au]", NativeKeywordField::Author),
        ("[ad]", NativeKeywordField::Affiliation),
        ("[journal]", NativeKeywordField::Journal),
        ("[jour]", NativeKeywordField::Journal),
    ];
    const PREFIX_FIELDS: [(&str, NativeKeywordField); 3] = [
        ("AUTH:", NativeKeywordField::Author),
        ("AFFILIATION:", NativeKeywordField::Affiliation),
        ("JOURNAL:", NativeKeywordField::Journal),
    ];

    for (tag, field) in SUFFIX_FIELDS {
        for (start, _) in keyword.match_indices('[') {
            let end = start + tag.len();
            if keyword
                .get(start..end)
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case(tag))
                && keyword[..start].chars().any(|ch| !ch.is_whitespace())
                && keyword.get(end..).is_some_and(|remainder| {
                    remainder
                        .chars()
                        .next()
                        .is_none_or(|next| next.is_whitespace() || matches!(next, '(' | ')'))
                })
            {
                return Some(field);
            }
        }
    }

    for (prefix, field) in PREFIX_FIELDS {
        for (start, _) in keyword.char_indices() {
            let end = start + prefix.len();
            if keyword
                .get(start..end)
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case(prefix))
                && (start == 0
                    || keyword[..start]
                        .chars()
                        .next_back()
                        .is_some_and(|previous| previous.is_whitespace() || previous == '('))
            {
                return Some(field);
            }
        }
    }

    None
}

pub(super) fn validate_search_filter_values(
    filters: &ArticleSearchFilters,
) -> Result<(), BioMcpError> {
    if let Some(article_type) = filters
        .article_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        normalize_article_type(article_type)?;
    }
    validate_query_inputs(filters.gene.as_deref(), filters.keyword.as_deref())?;
    if let Some(field) = filters.keyword.as_deref().and_then(native_keyword_field) {
        let guidance = match field {
            NativeKeywordField::Author => "Use --author for author searches.",
            NativeKeywordField::Journal => "Use --journal for journal searches.",
            NativeKeywordField::Affiliation => {
                "Remove the provider field wrapper and use ordinary unfielded -k/--keyword text."
            }
        };
        return Err(BioMcpError::InvalidArgument(format!(
            "-k/--keyword is provider-neutral and does not accept PubMed or Europe PMC field syntax. {guidance}"
        )));
    }
    Ok(())
}

pub(super) fn validate_required_search_filters(
    filters: &ArticleSearchFilters,
) -> Result<(), BioMcpError> {
    if filters.gene.is_none()
        && filters.disease.is_none()
        && filters.drug.is_none()
        && filters.variant.is_none()
        && filters.author.is_none()
        && filters.keyword.is_none()
        && filters.article_type.is_none()
        && !filters.open_access
    {
        return Err(BioMcpError::InvalidArgument(
            "At least one filter is required. Example: biomcp search article -g BRAF".into(),
        ));
    }
    Ok(())
}

pub(super) fn normalized_date_bounds(
    filters: &ArticleSearchFilters,
) -> Result<(Option<String>, Option<String>), BioMcpError> {
    let normalized_date_from = normalized_date_bound(filters.date_from.as_deref(), "--date-from")?;
    let normalized_date_to = normalized_date_bound(filters.date_to.as_deref(), "--date-to")?;
    if let (Some(from), Some(to)) = (
        normalized_date_from.as_deref(),
        normalized_date_to.as_deref(),
    ) && from > to
    {
        return Err(BioMcpError::InvalidArgument(
            "--date-from must be <= --date-to".into(),
        ));
    }
    Ok((normalized_date_from, normalized_date_to))
}

pub(super) fn has_article_type_filter(filters: &ArticleSearchFilters) -> bool {
    filters
        .article_type
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
}

pub(super) fn has_author_filter(filters: &ArticleSearchFilters) -> bool {
    filters
        .author
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
}

pub(super) fn has_keyword_query(filters: &ArticleSearchFilters) -> bool {
    filters
        .keyword
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
}

pub(super) fn parse_row_date(value: Option<&str>) -> Option<String> {
    let value = value.map(str::trim).filter(|v| !v.is_empty())?;
    let truncated = value.get(0..10).unwrap_or(value);
    match truncated.len() {
        4 => Some(format!("{truncated}-01-01")),
        7 => Some(format!("{truncated}-01")),
        _ => Some(truncated.to_string()),
    }
}

fn matches_optional_journal_filter(
    row_journal: Option<&str>,
    expected_journal: Option<&str>,
) -> bool {
    let Some(expected) = expected_journal
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return true;
    };
    let Some(actual) = row_journal.map(str::trim).filter(|value| !value.is_empty()) else {
        return false;
    };
    actual
        .to_ascii_lowercase()
        .contains(&expected.to_ascii_lowercase())
}

pub(super) fn matches_optional_date_filter(
    row_date: Option<&str>,
    date_from: Option<&str>,
    date_to: Option<&str>,
) -> bool {
    if date_from.is_none() && date_to.is_none() {
        return true;
    }
    let Some(value) = parse_row_date(row_date) else {
        return false;
    };
    if let Some(from) = date_from
        && value.as_str() < from
    {
        return false;
    }
    if let Some(to) = date_to
        && value.as_str() > to
    {
        return false;
    }
    true
}

pub(super) fn matches_result_filters(
    row: &ArticleSearchResult,
    filters: &ArticleSearchFilters,
    date_from: Option<&str>,
    date_to: Option<&str>,
) -> bool {
    if filters.no_preprints && row.journal.as_deref().is_some_and(is_preprint_journal) {
        return false;
    }
    if filters.exclude_retracted && row.is_retracted == Some(true) {
        return false;
    }
    if !matches_optional_journal_filter(row.journal.as_deref(), filters.journal.as_deref()) {
        return false;
    }
    if !matches_optional_date_filter(row.date.as_deref(), date_from, date_to) {
        return false;
    }
    true
}

#[cfg(test)]
mod tests;
