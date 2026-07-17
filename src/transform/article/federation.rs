//! Source-specific article mapping helpers for federated search and detail hydration.

use chrono::NaiveDate;

use crate::entities::article::{
    Article, ArticleAuthorCompleteness, ArticleSearchResult, ArticleSource,
};
use crate::sources::europepmc::EuropePmcResult;
use crate::sources::pubmed::ESummaryEntry;
use crate::sources::pubtator::{PubTatorDocument, PubTatorSearchResult};

use super::anchors::{
    article_search_abstract_snippet, clean_abstract, clean_title, normalize_article_search_text,
    truncate_abstract,
};

pub fn from_pubtator_document(doc: &PubTatorDocument) -> Article {
    let mut title: Option<String> = None;
    let mut abstract_text: Option<String> = None;
    for p in &doc.passages {
        let kind = p
            .infons
            .as_ref()
            .and_then(|i| i.kind.as_deref())
            .unwrap_or("");
        let text = p.text.as_deref().unwrap_or("").trim();
        if text.is_empty() {
            continue;
        }
        match kind {
            "title" if title.is_none() => title = Some(text.to_string()),
            "abstract" if abstract_text.is_none() => abstract_text = Some(text.to_string()),
            _ => {}
        }
    }

    Article {
        section_outcomes: crate::entities::section_outcome::SectionOutcomes::with_keys(
            crate::entities::article::ARTICLE_OUTCOME_KEYS,
        ),
        pmid: doc.pmid.map(|v| v.to_string()),
        pmcid: doc.pmcid.clone(),
        doi: None,
        title: title.unwrap_or_default().trim().to_string(),
        authors: doc.authors.clone(),
        author_count: doc.authors.len(),
        author_completeness: if doc.authors.is_empty() {
            ArticleAuthorCompleteness::Unavailable
        } else {
            ArticleAuthorCompleteness::Complete
        },
        author_source: ArticleSource::PubTator,
        journal: doc
            .journal
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        date: doc
            .date
            .as_deref()
            .and_then(|d| d.get(0..10))
            .map(|s| s.to_string()),
        citation_count: None,
        publication_type: None,
        open_access: None,
        abstract_text: abstract_text
            .map(|t| truncate_abstract(&t))
            .filter(|t| !t.is_empty()),
        full_text_path: None,
        full_text_note: None,
        full_text_source: None,
        full_text_manifest: None,
        not_included: None,
        europepmc_license: None,
        europepmc_retracted: None,
        annotations: None,
        indexing: None,
        semantic_scholar: None,
        pubtator_fallback: false,
    }
}

fn parse_citation_count(value: Option<&serde_json::Value>) -> Option<u64> {
    let value = value?;
    match value {
        serde_json::Value::Number(n) => n.as_u64(),
        serde_json::Value::String(s) => s.trim().parse::<u64>().ok(),
        _ => None,
    }
}

fn normalize_publication_type(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    let mapped = if lower.contains("meta-analysis") {
        "Meta-Analysis".to_string()
    } else if lower.contains("review") {
        "Review".to_string()
    } else if lower.contains("case report") {
        "Case Report".to_string()
    } else if lower.contains("research-article") || lower.contains("journal article") {
        "Research Article".to_string()
    } else {
        trimmed.to_string()
    };
    Some(mapped)
}

fn collect_publication_types_from_value(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::String(s) => {
            for token in s.split(';') {
                let token = token.trim();
                if !token.is_empty() {
                    out.push(token.to_string());
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                if let Some(text) = item.as_str().map(str::trim).filter(|v| !v.is_empty()) {
                    out.push(text.to_string());
                    continue;
                }
                if let Some(text) = item
                    .as_object()
                    .and_then(|o| o.get("name"))
                    .and_then(|v| v.as_str())
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                {
                    out.push(text.to_string());
                    continue;
                }
                collect_publication_types_from_value(item, out);
            }
        }
        serde_json::Value::Object(obj) => {
            for value in obj.values() {
                collect_publication_types_from_value(value, out);
            }
        }
        _ => {}
    };
}

fn publication_types(hit: &EuropePmcResult) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(value) = hit.pub_type.as_ref() {
        collect_publication_types_from_value(value, &mut out);
    }
    if let Some(value) = hit.pub_type_list.as_ref() {
        collect_publication_types_from_value(value, &mut out);
    }

    let mut deduped = Vec::new();
    for value in out {
        if deduped
            .iter()
            .any(|v: &String| v.eq_ignore_ascii_case(&value))
        {
            continue;
        }
        deduped.push(value);
    }
    deduped
}

fn parse_publication_type(hit: &EuropePmcResult) -> Option<String> {
    publication_types(hit)
        .into_iter()
        .find_map(|v| normalize_publication_type(&v))
}

fn retraction_status(hit: &EuropePmcResult) -> Option<bool> {
    let types = publication_types(hit);
    if types.is_empty() {
        return None;
    }
    Some(
        types
            .into_iter()
            .any(|value| value.to_ascii_lowercase().contains("retracted publication")),
    )
}

fn is_retracted_publication(hit: &EuropePmcResult) -> bool {
    retraction_status(hit).unwrap_or(false)
}

fn parse_open_access(value: Option<&serde_json::Value>) -> Option<bool> {
    let value = value?;
    match value {
        serde_json::Value::Bool(v) => Some(*v),
        serde_json::Value::String(v) => match v.trim().to_ascii_uppercase().as_str() {
            "Y" | "YES" | "TRUE" | "1" => Some(true),
            "N" | "NO" | "FALSE" | "0" => Some(false),
            _ => None,
        },
        serde_json::Value::Number(v) => v.as_u64().map(|n| n > 0),
        _ => None,
    }
}

fn split_author_string(value: &str) -> Vec<String> {
    let v = value.trim();
    if v.is_empty() {
        return vec![];
    }
    v.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn from_europepmc_result(hit: &EuropePmcResult) -> Article {
    let authors = hit
        .author_string
        .as_deref()
        .map(split_author_string)
        .unwrap_or_default();
    let author_completeness = if authors.is_empty() {
        ArticleAuthorCompleteness::Unavailable
    } else {
        ArticleAuthorCompleteness::SourceLimited
    };
    Article {
        section_outcomes: crate::entities::section_outcome::SectionOutcomes::with_keys(
            crate::entities::article::ARTICLE_OUTCOME_KEYS,
        ),
        pmid: hit.pmid.clone(),
        pmcid: hit.pmcid.clone(),
        doi: hit.doi.clone(),
        title: clean_title(hit.title.as_deref().unwrap_or_default()),
        author_count: authors.len(),
        authors,
        author_completeness,
        author_source: ArticleSource::EuropePmc,
        journal: hit
            .journal_title
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        date: hit
            .first_publication_date
            .as_ref()
            .or(hit.pub_year.as_ref())
            .map(|s| s.get(0..10).unwrap_or(s).to_string()),
        citation_count: parse_citation_count(hit.cited_by_count.as_ref()),
        publication_type: parse_publication_type(hit),
        open_access: parse_open_access(hit.is_open_access.as_ref()),
        abstract_text: hit
            .abstract_text
            .as_deref()
            .map(clean_abstract)
            .map(|text| truncate_abstract(&text))
            .filter(|text| !text.is_empty()),
        full_text_path: None,
        full_text_note: None,
        full_text_source: None,
        full_text_manifest: None,
        not_included: None,
        europepmc_license: hit
            .license
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        europepmc_retracted: retraction_status(hit),
        annotations: None,
        indexing: None,
        semantic_scholar: None,
        pubtator_fallback: false,
    }
}

pub fn merge_europepmc_metadata(article: &mut Article, hit: &EuropePmcResult) {
    if article.doi.is_none() {
        article.doi = hit.doi.clone();
    }
    if article.pmcid.is_none() {
        article.pmcid = hit.pmcid.clone();
    }
    if article.journal.is_none() {
        article.journal = hit
            .journal_title
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
    }
    if article.date.is_none() {
        article.date = hit
            .first_publication_date
            .as_ref()
            .or(hit.pub_year.as_ref())
            .map(|s| s.get(0..10).unwrap_or(s).to_string());
    }

    article.citation_count = parse_citation_count(hit.cited_by_count.as_ref());
    article.publication_type = parse_publication_type(hit);
    article.open_access = parse_open_access(hit.is_open_access.as_ref());
    article.europepmc_license = hit
        .license
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    article.europepmc_retracted = retraction_status(hit);
    if article.abstract_text.is_none() {
        article.abstract_text = hit
            .abstract_text
            .as_deref()
            .map(clean_abstract)
            .map(|text| truncate_abstract(&text))
            .filter(|text| !text.is_empty());
    }
}

pub fn from_europepmc_search_result(hit: &EuropePmcResult) -> Option<ArticleSearchResult> {
    let pmid = hit
        .pmid
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())?
        .to_string();
    let title = clean_title(hit.title.as_deref().unwrap_or_default());
    let abstract_text = hit.abstract_text.as_deref().map(clean_abstract);
    Some(ArticleSearchResult {
        pmid,
        pmcid: hit
            .pmcid
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        doi: hit
            .doi
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        arxiv_id: None,
        semantic_scholar_id: None,
        title,
        journal: hit
            .journal_title
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        date: hit
            .first_publication_date
            .as_ref()
            .or(hit.pub_year.as_ref())
            .map(|s| s.get(0..10).unwrap_or(s).to_string()),
        first_index_date: hit
            .first_index_date
            .as_deref()
            .and_then(parse_europepmc_index_date),
        citation_count: parse_citation_count(hit.cited_by_count.as_ref()),
        influential_citation_count: None,
        source: ArticleSource::EuropePmc,
        score: None,
        is_retracted: Some(is_retracted_publication(hit)),
        abstract_snippet: abstract_text
            .as_deref()
            .and_then(article_search_abstract_snippet),
        ranking: None,
        matched_sources: vec![ArticleSource::EuropePmc],
        normalized_title: normalize_article_search_text(hit.title.as_deref().unwrap_or_default()),
        normalized_abstract: abstract_text
            .as_deref()
            .map(normalize_article_search_text)
            .unwrap_or_default(),
        publication_type: parse_publication_type(hit),
        source_local_position: 0,
    })
}

pub fn from_pubtator_search_result(hit: &PubTatorSearchResult) -> Option<ArticleSearchResult> {
    let pmid = hit
        .pmid
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())?
        .to_string();
    Some(ArticleSearchResult {
        pmid,
        pmcid: None,
        doi: None,
        arxiv_id: None,
        semantic_scholar_id: None,
        title: clean_title(hit.title.as_deref().unwrap_or_default()),
        journal: hit
            .journal
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(|v| v.to_string()),
        date: hit
            .date
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(|v| v.get(0..10).unwrap_or(v).to_string()),
        first_index_date: None,
        citation_count: None,
        influential_citation_count: None,
        source: ArticleSource::PubTator,
        score: hit.score,
        is_retracted: None,
        abstract_snippet: None,
        ranking: None,
        matched_sources: vec![ArticleSource::PubTator],
        normalized_title: normalize_article_search_text(hit.title.as_deref().unwrap_or_default()),
        normalized_abstract: String::new(),
        publication_type: None,
        source_local_position: 0,
    })
}

fn parse_sortpubdate(value: &str) -> Option<String> {
    let prefix = value.trim().get(0..10)?;
    let bytes = prefix.as_bytes();
    if bytes[4] != b'/'
        || bytes[7] != b'/'
        || !bytes[0..4].iter().all(|byte| byte.is_ascii_digit())
        || !bytes[5..7].iter().all(|byte| byte.is_ascii_digit())
        || !bytes[8..10].iter().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    Some(prefix.replace('/', "-"))
}

fn parse_europepmc_index_date(value: &str) -> Option<NaiveDate> {
    let prefix = value.trim().get(0..10)?;
    NaiveDate::parse_from_str(prefix, "%Y-%m-%d").ok()
}

fn parse_pubmed_summary_date(value: &str) -> Option<NaiveDate> {
    let prefix = value.trim().get(0..10)?;
    NaiveDate::parse_from_str(prefix, "%Y/%m/%d").ok()
}

fn pubmed_month_number(value: &str) -> Option<&'static str> {
    match value
        .trim_matches(|ch: char| !ch.is_ascii_alphabetic())
        .to_ascii_lowercase()
        .as_str()
    {
        "jan" | "january" => Some("01"),
        "feb" | "february" => Some("02"),
        "mar" | "march" => Some("03"),
        "apr" | "april" => Some("04"),
        "may" => Some("05"),
        "jun" | "june" => Some("06"),
        "jul" | "july" => Some("07"),
        "aug" | "august" => Some("08"),
        "sep" | "sept" | "september" => Some("09"),
        "oct" | "october" => Some("10"),
        "nov" | "november" => Some("11"),
        "dec" | "december" => Some("12"),
        _ => None,
    }
}

fn parse_pubdate(value: &str) -> Option<String> {
    let mut parts = value.split_whitespace();
    let year = parts.next()?.trim();
    if year.len() != 4 || !year.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }

    let Some(month) = parts.next().and_then(pubmed_month_number) else {
        return Some(year.to_string());
    };
    let Some(day_token) = parts.next() else {
        return Some(format!("{year}-{month}"));
    };
    let day = day_token
        .trim_matches(|ch: char| !ch.is_ascii_digit())
        .parse::<u8>()
        .ok()
        .filter(|day| (1..=31).contains(day));
    match day {
        Some(day) => Some(format!("{year}-{month}-{day:02}")),
        None => Some(format!("{year}-{month}")),
    }
}

pub fn from_pubmed_esummary_entry(entry: &ESummaryEntry) -> Option<ArticleSearchResult> {
    let title = clean_title(&entry.title);
    if title.is_empty() {
        return None;
    }

    let journal = entry
        .fulljournalname
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .or_else(|| {
            entry
                .source
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| value.to_string())
        });
    let date = entry
        .sortpubdate
        .as_deref()
        .and_then(parse_sortpubdate)
        .or_else(|| entry.pubdate.as_deref().and_then(parse_pubdate));
    let first_index_date = entry
        .edat
        .as_deref()
        .and_then(parse_pubmed_summary_date)
        .or_else(|| entry.lr.as_deref().and_then(parse_pubmed_summary_date));

    Some(ArticleSearchResult {
        pmid: entry.uid.clone(),
        pmcid: None,
        doi: None,
        arxiv_id: None,
        semantic_scholar_id: None,
        title: title.clone(),
        journal,
        date,
        first_index_date,
        citation_count: None,
        influential_citation_count: None,
        source: ArticleSource::PubMed,
        matched_sources: vec![ArticleSource::PubMed],
        score: None,
        is_retracted: None,
        abstract_snippet: None,
        ranking: None,
        normalized_title: normalize_article_search_text(&title),
        normalized_abstract: String::new(),
        publication_type: None,
        source_local_position: 0,
    })
}

#[cfg(test)]
mod tests;
