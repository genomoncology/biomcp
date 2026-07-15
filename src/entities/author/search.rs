use super::*;
use crate::sources::semantic_scholar::{
    SemanticScholarAuthor, SemanticScholarAuthorSearchResponse,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorSearchQuery {
    pub name: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorPagination {
    pub offset: usize,
    pub limit: usize,
    pub total: Option<u64>,
    pub next: Option<usize>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorDegradation {
    pub code: &'static str,
    pub message: String,
    pub malformed_rows: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorSearchResult {
    pub identity: AuthorIdentity,
    pub display_name: String,
    pub affiliations: Vec<AuthorAssertion<String>>,
    pub paper_count: Option<u64>,
    pub citation_count: Option<u64>,
    pub h_index: Option<u64>,
    pub conflicts: Vec<AuthorConflict>,
    pub warnings: Vec<AuthorWarning>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorProviderBucket {
    pub source: &'static str,
    pub results: Vec<AuthorSearchResult>,
    pub pagination: AuthorPagination,
    pub status: ProviderStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degradation: Option<AuthorDegradation>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorSearchResponse {
    pub query: AuthorSearchQuery,
    pub providers: Vec<AuthorProviderBucket>,
    pub _meta: AuthorMeta,
}

pub(super) fn map_row(row: SemanticScholarAuthor, observed_at: &str) -> Option<AuthorSearchResult> {
    let value = valid_wire_id(row.author_id)?;
    let name = nonblank(row.name)?;
    let url = evidence_url(&value);
    let affiliations = row
        .affiliations
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| {
            let value = v.trim().to_string();
            (!value.is_empty()).then(|| AuthorAssertion {
                value,
                evidence: AuthorEvidence {
                    source: "semantic_scholar",
                    url: url.clone(),
                },
                temporal: TemporalAnchor::ObservedAt(observed_at.to_string()),
            })
        })
        .collect();
    Some(AuthorSearchResult {
        identity: AuthorIdentity::ExactProvider {
            id: provider_id(value),
        },
        display_name: name,
        affiliations,
        paper_count: row.paper_count,
        citation_count: row.citation_count,
        h_index: row.h_index,
        conflicts: vec![],
        warnings: vec![AuthorWarning::unresolved_orcid()],
    })
}

pub async fn search(
    name: &str,
    offset: usize,
    limit: usize,
) -> Result<AuthorSearchResponse, crate::error::BioMcpError> {
    let query = name.trim();
    if query.is_empty() {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "author search query is required".into(),
        ));
    }
    if limit == 0 || limit > 100 {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "author --limit must be between 1 and 100".into(),
        ));
    }
    let client = crate::sources::semantic_scholar::SemanticScholarClient::new()?;
    let response = client.author_search(query, offset, limit).await;
    Ok(map_response(
        query,
        offset,
        limit,
        response,
        &chrono::Utc::now().to_rfc3339(),
    ))
}

fn map_response(
    query: &str,
    offset: usize,
    limit: usize,
    response: Result<SemanticScholarAuthorSearchResponse, crate::error::BioMcpError>,
    observed_at: &str,
) -> AuthorSearchResponse {
    let (bucket, evidence_urls, next_commands) = match response {
        Ok(SemanticScholarAuthorSearchResponse {
            total, next, data, ..
        }) => {
            let row_count = data.len();
            let results: Vec<_> = data
                .into_iter()
                .filter_map(|r| map_row(r, observed_at))
                .collect();
            let malformed = row_count - results.len();
            let (next, overflow) = match next {
                Some(v) => match usize::try_from(v) {
                    Ok(v) => (Some(v), false),
                    Err(_) => (None, true),
                },
                None => (None, false),
            };
            let status = if malformed > 0 || overflow {
                ProviderStatus::Degraded
            } else {
                ProviderStatus::Available
            };
            let degradation = (status == ProviderStatus::Degraded).then(|| AuthorDegradation {
                code: "malformed_provider_response",
                message: match (malformed, overflow) {
                    (0, true) => "Semantic Scholar returned an unusable continuation".into(),
                    (count, true) => format!(
                        "Semantic Scholar returned {count} malformed author rows and an unusable continuation"
                    ),
                    (count, false) => {
                        debug_assert!(count > 0);
                        format!("Semantic Scholar returned {count} malformed author rows")
                    }
                },
                malformed_rows: malformed,
            });
            let evidence_urls = results
                .iter()
                .map(|r| {
                    let AuthorIdentity::ExactProvider { id } = &r.identity;
                    AuthorEvidenceUrl {
                        source: "semantic_scholar",
                        url: evidence_url(&id.value),
                    }
                })
                .collect::<Vec<_>>();
            let next_commands = results
                .iter()
                .map(|r| {
                    let AuthorIdentity::ExactProvider { id } = &r.identity;
                    format!("biomcp get author {id}")
                })
                .collect();
            (
                AuthorProviderBucket {
                    source: "semantic_scholar",
                    results,
                    pagination: AuthorPagination {
                        offset,
                        limit,
                        total,
                        next,
                    },
                    status,
                    degradation,
                },
                evidence_urls,
                next_commands,
            )
        }
        Err(err) => (
            AuthorProviderBucket {
                source: "semantic_scholar",
                results: vec![],
                pagination: AuthorPagination {
                    offset,
                    limit,
                    total: None,
                    next: None,
                },
                status: ProviderStatus::Unavailable,
                degradation: Some(AuthorDegradation {
                    code: "provider_unavailable",
                    message: format!(
                        "Semantic Scholar author search unavailable: {}",
                        sanitized_error(&err)
                    ),
                    malformed_rows: 0,
                }),
            },
            vec![],
            vec![],
        ),
    };
    let status = bucket.status;
    AuthorSearchResponse {
        query: AuthorSearchQuery {
            name: query.to_string(),
        },
        providers: vec![bucket],
        _meta: AuthorMeta {
            source_status: vec![AuthorSourceStatus {
                source: "semantic_scholar",
                status,
            }],
            evidence_urls,
            next_commands,
        },
    }
}
fn sanitized_error(err: &crate::error::BioMcpError) -> &'static str {
    match err {
        crate::error::BioMcpError::InvalidArgument(_) => "invalid request",
        _ => "provider request failed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: Option<&str>, name: Option<&str>) -> SemanticScholarAuthor {
        SemanticScholarAuthor {
            author_id: id.map(str::to_string),
            name: name.map(str::to_string),
            affiliations: Some(vec![" Lab ".into(), " ".into()]),
            ..Default::default()
        }
    }

    #[test]
    fn mapping_retains_valid_rows_and_degrades_malformed_rows() {
        let mut second = row(Some("2"), Some("Second"));
        second.paper_count = Some(42);
        let response = map_response(
            "Name",
            0,
            5,
            Ok(SemanticScholarAuthorSearchResponse {
                total: Some(3),
                offset: Some(0),
                next: Some(5),
                data: vec![
                    second,
                    row(None, Some("Missing ID")),
                    row(Some("1"), Some("First")),
                ],
            }),
            "2026-07-14T00:00:00Z",
        );
        let provider = &response.providers[0];
        assert_eq!(provider.status, ProviderStatus::Degraded);
        assert_eq!(provider.results.len(), 2);
        assert_eq!(provider.results[0].display_name, "Second");
        assert_eq!(provider.results[0].paper_count, Some(42));
        assert_eq!(provider.results[1].paper_count, None);
        assert_eq!(
            provider.results[0].warnings[0].code,
            "orcid_link_not_established"
        );
        assert_eq!(provider.degradation.as_ref().unwrap().malformed_rows, 1);
        assert_eq!(provider.results[0].affiliations.len(), 1);
        assert_eq!(
            provider.results[0].affiliations[0].temporal,
            TemporalAnchor::ObservedAt("2026-07-14T00:00:00Z".into())
        );
        assert_eq!(
            response._meta.next_commands,
            [
                "biomcp get author semanticscholar:2",
                "biomcp get author semanticscholar:1"
            ]
        );
    }

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn continuation_overflow_degrades_without_wrapping() {
        let response = map_response(
            "Name",
            0,
            5,
            Ok(SemanticScholarAuthorSearchResponse {
                total: Some(1),
                offset: Some(0),
                next: Some(u64::MAX),
                data: vec![row(Some("1"), Some("Name"))],
            }),
            "now",
        );
        let provider = &response.providers[0];
        assert_eq!(provider.status, ProviderStatus::Degraded);
        assert_eq!(provider.pagination.next, None);
        assert!(
            provider
                .degradation
                .as_ref()
                .unwrap()
                .message
                .contains("unusable continuation")
        );
    }

    #[test]
    fn mapping_distinguishes_healthy_empty_and_unavailable() {
        let empty = map_response(
            "Nobody",
            4,
            5,
            Ok(SemanticScholarAuthorSearchResponse {
                total: Some(0),
                offset: Some(4),
                next: None,
                data: vec![],
            }),
            "now",
        );
        assert_eq!(empty.providers[0].status, ProviderStatus::Available);
        assert!(empty.providers[0].results.is_empty());

        let unavailable = map_response(
            "Nobody",
            4,
            5,
            Err(crate::error::BioMcpError::Api {
                api: "semantic_scholar".into(),
                message: "secret provider body".into(),
            }),
            "now",
        );
        assert_eq!(unavailable.providers[0].status, ProviderStatus::Unavailable);
        let message = &unavailable.providers[0]
            .degradation
            .as_ref()
            .unwrap()
            .message;
        assert!(!message.contains("secret provider body"));
        assert!(unavailable._meta.evidence_urls.is_empty());
    }
}
