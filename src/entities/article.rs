use std::collections::HashSet;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::europepmc::{
    EuropePmcClient, EuropePmcResult, EuropePmcSearchResponse, EuropePmcSort,
};
use crate::sources::ncbi_idconv::NcbiIdConverterClient;
use crate::sources::pmc_oa::PmcOaClient;
use crate::sources::pubtator::PubTatorClient;
use crate::transform;
use crate::utils::date::validate_since;
use crate::utils::download;
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmcid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    pub title: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_access: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abstract_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_text_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_text_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ArticleAnnotations>,
    #[serde(default)]
    pub pubtator_fallback: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleAnnotations {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub genes: Vec<AnnotationCount>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diseases: Vec<AnnotationCount>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chemicals: Vec<AnnotationCount>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mutations: Vec<AnnotationCount>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnnotationCount {
    pub text: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleSearchResult {
    pub pmid: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_count: Option<u64>,
    #[serde(default)]
    pub is_retracted: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArticleSort {
    #[default]
    Date,
    Citations,
    Relevance,
}

impl ArticleSort {
    pub fn from_flag(value: &str) -> Result<Self, BioMcpError> {
        let value = value.trim();
        match value.to_ascii_lowercase().as_str() {
            "date" => Ok(Self::Date),
            "citations" => Ok(Self::Citations),
            "relevance" => Ok(Self::Relevance),
            _ => Err(BioMcpError::InvalidArgument(
                "Invalid article sort. Expected one of: date, citations, relevance".into(),
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Date => "date",
            Self::Citations => "citations",
            Self::Relevance => "relevance",
        }
    }

    fn as_europepmc_sort(self) -> EuropePmcSort {
        match self {
            Self::Date => EuropePmcSort::Date,
            Self::Citations => EuropePmcSort::Citations,
            Self::Relevance => EuropePmcSort::Relevance,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArticleSearchFilters {
    pub gene: Option<String>,
    pub disease: Option<String>,
    pub drug: Option<String>,
    pub author: Option<String>,
    pub keyword: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub article_type: Option<String>,
    pub journal: Option<String>,
    pub open_access: bool,
    pub no_preprints: bool,
    pub exclude_retracted: bool,
    pub sort: ArticleSort,
}

const ARTICLE_SECTION_ANNOTATIONS: &str = "annotations";
const ARTICLE_SECTION_FULLTEXT: &str = "fulltext";
const ARTICLE_SECTION_ALL: &str = "all";

pub const ARTICLE_SECTION_NAMES: &[&str] = &[
    ARTICLE_SECTION_ANNOTATIONS,
    ARTICLE_SECTION_FULLTEXT,
    ARTICLE_SECTION_ALL,
];

fn is_doi(id: &str) -> bool {
    let id = id.trim();
    if !id.starts_with("10.") {
        return false;
    }
    id.contains('/')
}

fn parse_pmid(id: &str) -> Option<u32> {
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    if !id.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    id.parse::<u32>().ok()
}

fn parse_pmcid(id: &str) -> Option<String> {
    let id = id.trim();
    if id.len() < 4 {
        return None;
    }
    let (prefix, rest) = id.split_at(3);
    if !prefix.eq_ignore_ascii_case("PMC") {
        return None;
    }
    let rest = rest.trim();
    if rest.is_empty() || !rest.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!("PMC{rest}"))
}

#[derive(Debug, Clone)]
enum ArticleIdType {
    Pmc(String),
    Doi(String),
    Pmid(u32),
    Invalid,
}

fn parse_article_id(id: &str) -> ArticleIdType {
    let id = id.trim();
    if let Some(pmcid) = parse_pmcid(id) {
        return ArticleIdType::Pmc(pmcid);
    }
    if is_doi(id) {
        return ArticleIdType::Doi(id.to_string());
    }
    if let Some(pmid) = parse_pmid(id) {
        return ArticleIdType::Pmid(pmid);
    }
    ArticleIdType::Invalid
}

fn is_preprint_journal(journal: &str) -> bool {
    let j = journal.to_ascii_lowercase();
    j.contains("biorxiv") || j.contains("medrxiv") || j.contains("arxiv")
}

fn europepmc_escape(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        return String::new();
    }

    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        if matches!(
            ch,
            '\\' | '\"'
                | '+'
                | '-'
                | '!'
                | '('
                | ')'
                | '{'
                | '}'
                | '['
                | ']'
                | '^'
                | '~'
                | '*'
                | '?'
                | ':'
                | '|'
        ) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }

    escaped
}

fn europepmc_phrase(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        return String::new();
    }
    let escaped = europepmc_escape(value);
    if value.chars().any(|c| c.is_whitespace()) || value.contains('/') {
        format!("\"{escaped}\"")
    } else {
        escaped
    }
}

fn europepmc_keyword(value: &str) -> String {
    europepmc_escape(value)
}

fn normalize_article_type(value: &str) -> Result<&'static str, BioMcpError> {
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

fn build_search_query(filters: &ArticleSearchFilters) -> Result<String, BioMcpError> {
    if filters.gene.is_none()
        && filters.disease.is_none()
        && filters.drug.is_none()
        && filters.author.is_none()
        && filters.keyword.is_none()
        && filters.article_type.is_none()
        && !filters.open_access
    {
        return Err(BioMcpError::InvalidArgument(
            "At least one filter is required. Example: biomcp search article -g BRAF".into(),
        ));
    }

    let normalized_date_from = filters
        .date_from
        .as_deref()
        .map(validate_since)
        .transpose()?;
    let normalized_date_to = filters.date_to.as_deref().map(validate_since).transpose()?;
    if let (Some(from), Some(to)) = (
        normalized_date_from.as_deref(),
        normalized_date_to.as_deref(),
    ) && from > to
    {
        return Err(BioMcpError::InvalidArgument(
            "--date-from must be <= --date-to".into(),
        ));
    }
    let mut terms: Vec<String> = Vec::new();

    if let Some(gene) = filters
        .gene
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(europepmc_phrase(gene));
    }
    if let Some(disease) = filters
        .disease
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(europepmc_phrase(disease));
    }
    if let Some(drug) = filters
        .drug
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(europepmc_phrase(drug));
    }
    if let Some(author) = filters
        .author
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(format!("AUTH:{}", europepmc_phrase(author)));
    }
    if let Some(keyword) = filters
        .keyword
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(europepmc_keyword(keyword));
    }

    if let Some(article_type) = filters
        .article_type
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let normalized = normalize_article_type(article_type)?;
        terms.push(format!("PUB_TYPE:\"{normalized}\""));
    }

    if let Some(journal) = filters
        .journal
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(format!("JOURNAL:{}", europepmc_phrase(journal)));
    }

    if filters.open_access {
        terms.push("OPEN_ACCESS:y".into());
    }

    if let Some(from) = normalized_date_from.as_deref() {
        let to = normalized_date_to.as_deref().unwrap_or("*");
        terms.push(format!("FIRST_PDATE:[{from} TO {to}]"));
    } else if let Some(to) = normalized_date_to.as_deref() {
        terms.push(format!("FIRST_PDATE:[* TO {to}]"));
    }

    if filters.no_preprints {
        terms.push("NOT SRC:PPR".into());
    }
    if filters.exclude_retracted {
        terms.push("NOT PUB_TYPE:\"retracted publication\"".into());
    }

    Ok(terms.join(" AND "))
}

#[derive(Debug, Clone, Copy, Default)]
struct ArticleSections {
    include_annotations: bool,
    include_fulltext: bool,
    include_all: bool,
}

fn parse_sections(sections: &[String]) -> Result<ArticleSections, BioMcpError> {
    let mut out = ArticleSections::default();

    for raw in sections {
        let section = raw.trim().to_ascii_lowercase();
        if section.is_empty() {
            continue;
        }
        if section == "--json" || section == "-j" {
            continue;
        }

        match section.as_str() {
            ARTICLE_SECTION_ANNOTATIONS => out.include_annotations = true,
            ARTICLE_SECTION_FULLTEXT => out.include_fulltext = true,
            ARTICLE_SECTION_ALL => out.include_all = true,
            _ => {
                return Err(BioMcpError::InvalidArgument(format!(
                    "Unknown section \"{section}\" for article. Available: {}",
                    ARTICLE_SECTION_NAMES.join(", ")
                )));
            }
        }
    }

    if out.include_all {
        out.include_annotations = true;
        out.include_fulltext = true;
    }

    Ok(out)
}

fn is_section_only_request(sections: &[String], include_all: bool) -> bool {
    if include_all {
        return false;
    }
    sections.iter().any(|s| {
        let value = s.trim().to_ascii_lowercase();
        !value.is_empty() && value != "--json" && value != "-j"
    })
}

fn article_not_found(id: &str, suggestion_id: &str) -> BioMcpError {
    BioMcpError::NotFound {
        entity: "article".into(),
        id: id.to_string(),
        suggestion: format!("Try searching: biomcp search article -q \"{suggestion_id}\""),
    }
}

fn first_europepmc_hit(search: EuropePmcSearchResponse) -> Option<EuropePmcResult> {
    search.result_list.and_then(|l| l.result.into_iter().next())
}

fn is_pubtator_lag_error(err: &BioMcpError) -> bool {
    matches!(
        err,
        BioMcpError::Api { api, message }
            if api == "pubtator3" && (message.contains("HTTP 400") || message.contains("HTTP 404"))
    )
}

async fn resolve_article_from_pmid(
    pmid: u32,
    not_found_id: &str,
    suggestion_id: &str,
    pubtator: &PubTatorClient,
    europe: &EuropePmcClient,
    europe_hint: Option<&EuropePmcResult>,
) -> Result<Article, BioMcpError> {
    match pubtator.export_biocjson(pmid).await {
        Ok(resp) => {
            let doc = resp
                .documents
                .into_iter()
                .next()
                .ok_or_else(|| article_not_found(not_found_id, suggestion_id))?;

            let mut article = transform::article::from_pubtator_document(&doc);
            if let Some(hit) = europe_hint {
                transform::article::merge_europepmc_metadata(&mut article, hit);
            } else if let Ok(search) = europe.search_by_pmid(&pmid.to_string()).await
                && let Some(hit) = first_europepmc_hit(search)
            {
                transform::article::merge_europepmc_metadata(&mut article, &hit);
            }
            article.annotations = transform::article::extract_annotations(&doc);
            Ok(article)
        }
        Err(err) => {
            if !is_pubtator_lag_error(&err) {
                return Err(err);
            }

            let hit = match europe_hint.cloned() {
                Some(hit) => hit,
                None => {
                    let search = europe.search_by_pmid(&pmid.to_string()).await?;
                    first_europepmc_hit(search)
                        .ok_or_else(|| article_not_found(not_found_id, suggestion_id))?
                }
            };
            let mut article = transform::article::from_europepmc_result(&hit);
            article.pubtator_fallback = true;
            Ok(article)
        }
    }
}

pub async fn search(
    filters: &ArticleSearchFilters,
    limit: usize,
) -> Result<Vec<ArticleSearchResult>, BioMcpError> {
    Ok(search_page(filters, limit, 0).await?.results)
}

pub async fn search_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<ArticleSearchResult>, BioMcpError> {
    const MAX_SEARCH_LIMIT: usize = 50;
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }

    let europe = EuropePmcClient::new()?;
    let query = build_search_query(filters)?;
    let europepmc_sort = filters.sort.as_europepmc_sort();

    let mut out: Vec<ArticleSearchResult> = Vec::with_capacity(limit.min(10));
    let mut seen_pmids: HashSet<String> = HashSet::with_capacity(limit.min(10));
    let mut total: Option<usize> = None;
    let mut page: usize = if limit == 0 { 1 } else { (offset / limit) + 1 };
    let mut local_skip = if limit == 0 { 0 } else { offset % limit };
    while out.len() < limit {
        let page_size = (limit.saturating_sub(out.len())).clamp(1, 25);
        let resp = europe
            .search_query_with_sort(&query, page, page_size, europepmc_sort)
            .await?;
        if total.is_none() {
            total = resp.hit_count.map(|v| v as usize);
        }
        let Some(results) = resp.result_list.map(|v| v.result) else {
            break;
        };
        if results.is_empty() {
            break;
        }

        for hit in results {
            if local_skip > 0 {
                local_skip -= 1;
                continue;
            }
            if filters.no_preprints
                && hit
                    .journal_title
                    .as_deref()
                    .is_some_and(is_preprint_journal)
            {
                continue;
            }

            let Some(row) = transform::article::from_europepmc_search_result(&hit) else {
                continue;
            };
            if filters.exclude_retracted && row.is_retracted {
                continue;
            }
            if !seen_pmids.insert(row.pmid.clone()) {
                continue;
            }
            out.push(row);
            if out.len() >= limit {
                break;
            }
        }

        page += 1;
        if page > 20 {
            break;
        }
    }

    // Safety-first default: when date-sorted results contain no visible retraction marker,
    // try adding one matched retracted publication if available.
    if !filters.exclude_retracted
        && filters.sort == ArticleSort::Date
        && !out.iter().any(|row| row.is_retracted)
    {
        let retracted_query = format!("({query}) AND PUB_TYPE:\"retracted publication\"");
        if let Ok(resp) = europe
            .search_query_with_sort(&retracted_query, 1, 10, europepmc_sort)
            .await
        {
            let replacement = resp
                .result_list
                .map(|v| v.result)
                .unwrap_or_default()
                .into_iter()
                .filter_map(|hit| transform::article::from_europepmc_search_result(&hit))
                .find(|row| row.is_retracted && !seen_pmids.contains(&row.pmid));
            if let Some(row) = replacement {
                if out.len() >= limit && !out.is_empty() {
                    out.pop();
                }
                if out.len() < limit {
                    seen_pmids.insert(row.pmid.clone());
                    out.push(row);
                }
            }
        }
    }

    Ok(SearchPage::offset(out, total))
}

pub async fn get(id: &str, sections: &[String]) -> Result<Article, BioMcpError> {
    let id = id.trim();
    if id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "ID is required. Example: biomcp get article 22663011".into(),
        ));
    }
    if id.len() > 512 {
        return Err(BioMcpError::InvalidArgument("ID is too long.".into()));
    }

    let section_flags = parse_sections(sections)?;
    let full_text = section_flags.include_fulltext;
    let section_only = is_section_only_request(sections, section_flags.include_all);

    let pubtator = PubTatorClient::new()?;
    let europe = EuropePmcClient::new()?;

    let mut article = match parse_article_id(id) {
        ArticleIdType::Pmid(pmid) => {
            resolve_article_from_pmid(pmid, id, id, &pubtator, &europe, None).await?
        }
        ArticleIdType::Doi(doi) => {
            let search = europe.search_by_doi(&doi).await?;
            if search.hit_count.unwrap_or(0) == 0 {
                return Err(article_not_found(&doi, id));
            }
            let hit = first_europepmc_hit(search).ok_or_else(|| article_not_found(&doi, id))?;

            if let Some(pmid) = hit.pmid.as_deref().and_then(parse_pmid) {
                resolve_article_from_pmid(pmid, &doi, id, &pubtator, &europe, Some(&hit)).await?
            } else {
                transform::article::from_europepmc_result(&hit)
            }
        }
        ArticleIdType::Pmc(pmcid) => {
            let search = europe.search_by_pmcid(&pmcid).await?;
            if search.hit_count.unwrap_or(0) == 0 {
                return Err(article_not_found(&pmcid, id));
            }
            let hit = first_europepmc_hit(search).ok_or_else(|| article_not_found(&pmcid, id))?;

            if let Some(pmid) = hit.pmid.as_deref().and_then(parse_pmid) {
                resolve_article_from_pmid(pmid, &pmcid, id, &pubtator, &europe, Some(&hit)).await?
            } else {
                transform::article::from_europepmc_result(&hit)
            }
        }
        ArticleIdType::Invalid => {
            return Err(BioMcpError::InvalidArgument(
                "ID must be a PMID (digits), PMCID (starts with PMC), or DOI (starts with 10.). Example: biomcp get article 22663011".into(),
            ));
        }
    };

    if section_only && !section_flags.include_annotations {
        article.annotations = None;
    }

    if full_text {
        let mut full_text_err: Option<BioMcpError> = None;
        let mut resolved_pmcid = article.pmcid.clone();

        if resolved_pmcid.is_none() {
            match NcbiIdConverterClient::new() {
                Ok(ncbi) => {
                    if let Some(pmid) = article.pmid.as_deref() {
                        match ncbi.pmid_to_pmcid(pmid).await {
                            Ok(v) => resolved_pmcid = v,
                            Err(err) => full_text_err = Some(err),
                        }
                    } else if let Some(doi) = article.doi.as_deref() {
                        match ncbi.doi_to_pmcid(doi).await {
                            Ok(v) => resolved_pmcid = v,
                            Err(err) => full_text_err = Some(err),
                        }
                    }
                }
                Err(err) => full_text_err = Some(err),
            }
        }

        if article.pmcid.is_none() {
            article.pmcid = resolved_pmcid.clone();
        }

        let mut xml: Option<String> = None;
        if let Some(pmcid) = resolved_pmcid.as_deref() {
            match europe.get_full_text_xml("PMC", pmcid).await {
                Ok(v) => xml = v,
                Err(err) => full_text_err = Some(err),
            }
        }
        if xml.is_none()
            && let Some(pmcid) = resolved_pmcid.as_deref()
        {
            match PmcOaClient::new() {
                Ok(pmc_oa) => match pmc_oa.get_full_text_xml(pmcid).await {
                    Ok(v) => xml = v,
                    Err(err) => full_text_err = Some(err),
                },
                Err(err) => full_text_err = Some(err),
            }
        }
        if xml.is_none()
            && let Some(pmid) = article.pmid.as_deref()
        {
            match europe.get_full_text_xml("MED", pmid).await {
                Ok(v) => xml = v,
                Err(err) => full_text_err = Some(err),
            }
        }

        if let Some(xml) = xml {
            let text = transform::article::extract_text_from_xml(&xml);
            let key = article
                .pmid
                .as_deref()
                .or(article.doi.as_deref())
                .or(article.pmcid.as_deref())
                .unwrap_or(id);
            let path = download::save_atomic(key, &text).await?;
            article.full_text_path = Some(path);
            article.full_text_note = None;
        } else if let Some(err) = full_text_err {
            warn!(?err, id, "Full text retrieval failed");
            article.full_text_note = Some("Full text not available: API error".into());
        } else if article.pmcid.is_none() {
            article.full_text_note =
                Some("Full text not available: Article not in PubMed Central".into());
        } else {
            article.full_text_note =
                Some("Full text not available: Full text not available from Europe PMC".into());
        }
    }

    Ok(article)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_filters() -> ArticleSearchFilters {
        ArticleSearchFilters {
            gene: None,
            disease: None,
            drug: None,
            author: None,
            keyword: None,
            date_from: None,
            date_to: None,
            article_type: None,
            journal: None,
            open_access: false,
            no_preprints: false,
            exclude_retracted: false,
            sort: ArticleSort::Date,
        }
    }

    #[test]
    fn is_doi_basic() {
        assert!(is_doi("10.1056/NEJMoa1203421"));
        assert!(is_doi("10.1056/nejmoa1203421"));
        assert!(!is_doi("22663011"));
        assert!(!is_doi("doi:10.1056/NEJMoa1203421"));
    }

    #[test]
    fn parse_pmid_basic() {
        assert_eq!(parse_pmid("22663011"), Some(22663011));
        assert_eq!(parse_pmid(" 22663011 "), Some(22663011));
        assert_eq!(parse_pmid(""), None);
        assert_eq!(parse_pmid("10.1056/NEJMoa1203421"), None);
        assert_eq!(parse_pmid("abc"), None);
    }

    #[test]
    fn parse_pmcid_basic() {
        assert_eq!(parse_pmcid("PMC9984800"), Some("PMC9984800".into()));
        assert_eq!(parse_pmcid("pmc9984800"), Some("PMC9984800".into()));
        assert_eq!(parse_pmcid(" PMC9984800 "), Some("PMC9984800".into()));
        assert_eq!(parse_pmcid("PMC"), None);
        assert_eq!(parse_pmcid("PMCX"), None);
        assert_eq!(parse_pmcid("PMC-123"), None);
        assert_eq!(parse_pmcid("22663011"), None);
    }

    #[test]
    fn parse_article_id_basic() {
        match parse_article_id("PMC9984800") {
            ArticleIdType::Pmc(v) => assert_eq!(v, "PMC9984800"),
            _ => panic!("expected PMCID"),
        }
        match parse_article_id("10.1056/NEJMoa1203421") {
            ArticleIdType::Doi(v) => assert_eq!(v, "10.1056/NEJMoa1203421"),
            _ => panic!("expected DOI"),
        }
        match parse_article_id("22663011") {
            ArticleIdType::Pmid(v) => assert_eq!(v, 22663011),
            _ => panic!("expected PMID"),
        }
        assert!(matches!(
            parse_article_id("doi:10.1056/NEJMoa1203421"),
            ArticleIdType::Invalid
        ));
    }

    #[test]
    fn europepmc_keyword_does_not_quote_whitespace() {
        let term = europepmc_keyword("large language model clinical trials");
        assert_eq!(term, "large language model clinical trials");
    }

    #[test]
    fn build_search_query_keeps_phrase_quoting_for_entity_filters() {
        let mut filters = empty_filters();
        filters.gene = Some("BRAF V600E".into());
        filters.author = Some("Jane Doe".into());

        let query = build_search_query(&filters).expect("query should build");
        assert!(query.contains("\"BRAF V600E\""));
        assert!(query.contains("AUTH:\"Jane Doe\""));
    }

    #[test]
    fn build_search_query_combines_keyword_and_since() {
        let mut filters = empty_filters();
        filters.gene = Some("BRAF".into());
        filters.keyword = Some("large language model".into());
        filters.date_from = Some("2024-01-01".into());
        filters.no_preprints = true;

        let query = build_search_query(&filters).expect("query should build");
        assert!(query.contains("BRAF"));
        assert!(query.contains("large language model"));
        assert!(query.contains("FIRST_PDATE:[2024-01-01 TO *]"));
        assert!(query.contains("NOT SRC:PPR"));
    }

    #[test]
    fn build_search_query_excludes_retracted_when_requested() {
        let mut filters = empty_filters();
        filters.gene = Some("BRAF".into());
        filters.exclude_retracted = true;
        let query = build_search_query(&filters).expect("query should build");
        assert!(query.contains("NOT PUB_TYPE:\"retracted publication\""));
    }

    #[test]
    fn normalize_article_type_accepts_aliases() {
        assert_eq!(
            normalize_article_type("review").expect("review should normalize"),
            "review"
        );
        assert_eq!(
            normalize_article_type("research").expect("research alias should normalize"),
            "research-article"
        );
        assert_eq!(
            normalize_article_type("research-article").expect("research-article should normalize"),
            "research-article"
        );
        assert_eq!(
            normalize_article_type("case-reports").expect("case-reports should normalize"),
            "case-reports"
        );
        assert_eq!(
            normalize_article_type("metaanalysis").expect("metaanalysis alias should normalize"),
            "meta-analysis"
        );
    }

    #[test]
    fn build_search_query_rejects_unknown_article_type() {
        let mut filters = empty_filters();
        filters.gene = Some("BRAF".into());
        filters.article_type = Some("invalid".into());

        let err = build_search_query(&filters).expect_err("invalid article type should fail");
        let msg = err.to_string();
        assert!(msg.contains("Invalid argument"));
        assert!(msg.contains("case-reports"));
    }

    #[test]
    fn article_sort_parses_supported_values() {
        assert_eq!(
            ArticleSort::from_flag("date").expect("date should parse"),
            ArticleSort::Date
        );
        assert_eq!(
            ArticleSort::from_flag("citations").expect("citations should parse"),
            ArticleSort::Citations
        );
        assert_eq!(
            ArticleSort::from_flag("relevance").expect("relevance should parse"),
            ArticleSort::Relevance
        );
        assert!(ArticleSort::from_flag("newest").is_err());
    }

    #[test]
    fn pubtator_lag_error_is_400_or_404_only() {
        let err_400 = BioMcpError::Api {
            api: "pubtator3".into(),
            message: "HTTP 400 Bad Request: pending".into(),
        };
        let err_404 = BioMcpError::Api {
            api: "pubtator3".into(),
            message: "HTTP 404 Not Found: pending".into(),
        };
        let err_500 = BioMcpError::Api {
            api: "pubtator3".into(),
            message: "HTTP 500 Internal Server Error".into(),
        };
        let other_api_400 = BioMcpError::Api {
            api: "europepmc".into(),
            message: "HTTP 400 Bad Request".into(),
        };

        assert!(is_pubtator_lag_error(&err_400));
        assert!(is_pubtator_lag_error(&err_404));
        assert!(!is_pubtator_lag_error(&err_500));
        assert!(!is_pubtator_lag_error(&other_api_400));
    }
}
