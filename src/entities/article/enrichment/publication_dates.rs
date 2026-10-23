//! Correct future issue dates from publication metadata before candidate ranking.

use std::collections::{HashMap, HashSet};
use std::future::Future;

use chrono::{NaiveDate, Utc};

use crate::sources::pubmed::PubMedClient;

use super::{
    ArticleSearchResult, ArticleSource, ArticleSourceAvailability, ArticleSourceStatus,
    article_search_deadline_status, plain_article_search_deadline_elapsed,
};
use crate::entities::article::PUBMED_PAGE_SIZE;
use crate::entities::article::filters::parse_row_date;

 tokio::task_local! {
    static TODAY: NaiveDate;
}

pub(in crate::entities::article) fn today() -> NaiveDate {
    TODAY.try_with(|date| *date).unwrap_or_else(|_| Utc::now().date_naive())
}

pub(super) async fn with_today<T>(future: impl Future<Output = T>) -> T {
    TODAY.scope(Utc::now().date_naive(), future).await
}

fn date_key(value: Option<&str>) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(&parse_row_date(value)?, "%Y-%m-%d").ok()
}

fn failed(error: &crate::error::BioMcpError) -> ArticleSourceStatus {
    let failure = error.external_failure(
        crate::error::SourceProvider::PUBMED,
        "article publication date recovery",
    );
    ArticleSourceStatus {
        source: ArticleSource::PubMed,
        enabled: true,
        auth_mode: None,
        status: Some(ArticleSourceAvailability::Degraded),
        message: Some(format!("article publication date recovery: {}", failure.message)),
    }
}

pub(super) async fn enrich(rows: &mut [ArticleSearchResult]) -> Vec<ArticleSourceStatus> {
    let today = today();
    let mut seen = HashSet::new();
    let ids: Vec<String> = rows.iter().filter(|row| {
        date_key(row.date.as_deref()).is_some_and(|date| date > today)
            && !row.pmid.is_empty()
            && row.pmid.bytes().all(|byte| byte.is_ascii_digit())
            && seen.insert(row.pmid.clone())
    }).map(|row| row.pmid.clone()).collect();
    if ids.is_empty() {
        return Vec::new();
    }
    let expired = || vec![article_search_deadline_status(
        ArticleSource::PubMed, "publication date recovery",
    )];
    if plain_article_search_deadline_elapsed(None) {
        return expired();
    }
    let deadline = crate::sources::current_variant_article_deadline();
    let client = match match deadline.as_ref() {
        Some(deadline) => PubMedClient::new_with_deadline(deadline).await,
        None => PubMedClient::new(),
    } {
        Ok(client) => client,
        Err(error) => return vec![failed(&error)],
    };
    // A known publication date from another search provider remains usable if
    // PubMed has no electronic date. Every duplicate receives one selected date.
    let mut dates: HashMap<String, (NaiveDate, String)> = HashMap::new();
    for row in rows.iter().filter(|row| seen.contains(&row.pmid)) {
        retain_published_date(&mut dates, &row.pmid, row.date.as_deref(), today);
    }
    let mut statuses = Vec::new();
    for chunk in ids.chunks(PUBMED_PAGE_SIZE) {
        if plain_article_search_deadline_elapsed(None) {
            statuses.extend(expired());
            break;
        }
        match client.esummary(chunk).await {
            Ok(entries) => for entry in entries {
                if let Some(row) = crate::transform::article::from_pubmed_esummary_entry(&entry) {
                    retain_published_date(&mut dates, &row.pmid, row.date.as_deref(), today);
                }
            },
            Err(error) => statuses.push(failed(&error)),
        }
    }
    for row in rows {
        if let Some((_, date)) = dates.get(&row.pmid) {
            row.date = Some(date.clone());
        }
    }
    statuses
}

fn retain_published_date(
    dates: &mut HashMap<String, (NaiveDate, String)>,
    pmid: &str,
    value: Option<&str>,
    today: NaiveDate,
) {
    let Some(key) = date_key(value).filter(|date| *date <= today) else { return };
    if dates.get(pmid).is_none_or(|(existing, _)| key < *existing) {
        dates.insert(pmid.to_string(), (key, value.unwrap().to_string()));
    }
}
