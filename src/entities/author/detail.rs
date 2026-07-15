use super::*;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProviderAuthorRecord {
    pub id: ProviderAuthorId,
    pub source: &'static str,
    pub status: ProviderStatus,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorDetail {
    pub identity: AuthorIdentity,
    pub display_name: String,
    pub provider_records: Vec<ProviderAuthorRecord>,
    pub affiliations: Vec<AuthorAssertion<String>>,
    pub paper_count: Option<u64>,
    pub citation_count: Option<u64>,
    pub h_index: Option<u64>,
    pub conflicts: Vec<AuthorConflict>,
    pub warnings: Vec<AuthorWarning>,
    pub _meta: AuthorMeta,
}

pub async fn detail(raw_id: &str) -> Result<AuthorDetail, crate::error::BioMcpError> {
    let requested: ProviderAuthorId = raw_id.parse()?;
    let row = crate::sources::semantic_scholar::SemanticScholarClient::new()?
        .author_detail(&requested.value)
        .await
        .map_err(sanitized_detail_error)?;
    map_detail(row, &requested, &chrono::Utc::now().to_rfc3339())
}

fn map_detail(
    row: crate::sources::semantic_scholar::SemanticScholarAuthor,
    requested: &ProviderAuthorId,
    observed_at: &str,
) -> Result<AuthorDetail, crate::error::BioMcpError> {
    let returned = valid_wire_id(row.author_id).ok_or_else(contract_error)?;
    if returned != requested.value {
        return Err(contract_error());
    }
    let display_name = nonblank(row.name).ok_or_else(contract_error)?;
    let url = evidence_url(&returned);
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
    let id = provider_id(returned);
    Ok(AuthorDetail {
        identity: AuthorIdentity::ExactProvider { id: id.clone() },
        display_name,
        provider_records: vec![ProviderAuthorRecord {
            id,
            source: "semantic_scholar",
            status: ProviderStatus::Available,
        }],
        affiliations,
        paper_count: row.paper_count,
        citation_count: row.citation_count,
        h_index: row.h_index,
        conflicts: vec![],
        warnings: vec![AuthorWarning::unresolved_orcid()],
        _meta: AuthorMeta {
            source_status: vec![AuthorSourceStatus {
                source: "semantic_scholar",
                status: ProviderStatus::Available,
            }],
            evidence_urls: vec![AuthorEvidenceUrl {
                source: "semantic_scholar",
                url,
            }],
            next_commands: vec![],
        },
    })
}
fn contract_error() -> crate::error::BioMcpError {
    crate::error::BioMcpError::Api {
        api: "semantic_scholar".into(),
        message: "author detail response did not match the requested provider record".into(),
    }
}

fn sanitized_detail_error(_: crate::error::BioMcpError) -> crate::error::BioMcpError {
    crate::error::BioMcpError::Api {
        api: "semantic_scholar".into(),
        message: "author detail is unavailable; retry later".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::semantic_scholar::SemanticScholarAuthor;

    fn row(id: Option<&str>, name: Option<&str>) -> SemanticScholarAuthor {
        SemanticScholarAuthor {
            author_id: id.map(str::to_string),
            name: name.map(str::to_string),
            affiliations: Some(vec!["Institute".into()]),
            external_ids: Some(serde_json::Map::from_iter([(
                "ORCID".into(),
                serde_json::json!("private-sentinel"),
            )])),
            paper_count: Some(548),
            citation_count: Some(50_000),
            h_index: Some(100),
        }
    }

    #[test]
    fn detail_requires_matching_decimal_id_and_nonblank_name() {
        let requested: ProviderAuthorId = "semanticscholar:1".parse().unwrap();
        for invalid in [
            row(None, Some("Name")),
            row(Some("2"), Some("Name")),
            row(Some("1"), Some(" ")),
        ] {
            assert!(map_detail(invalid, &requested, "now").is_err());
        }
    }

    #[test]
    fn provider_failure_does_not_expose_response_body() {
        let error = sanitized_detail_error(crate::error::BioMcpError::Api {
            api: "semantic_scholar".into(),
            message: "HTTP 500: private-author@example.invalid fixture-private-profile".into(),
        });
        let rendered = error.to_string();
        assert!(rendered.contains("retry later"));
        assert!(!rendered.contains("private-author@example.invalid"));
        assert!(!rendered.contains("fixture-private-profile"));
    }

    #[test]
    fn detail_serialization_is_allowlisted_and_has_required_metadata_arrays() {
        let requested: ProviderAuthorId = "semanticscholar:1".parse().unwrap();
        let detail = map_detail(row(Some("1"), Some("Name")), &requested, "now").unwrap();
        let value = serde_json::to_value(detail).unwrap();
        assert_eq!(value["identity"]["id"], "semanticscholar:1");
        assert_eq!(value["provider_records"][0]["source"], "semantic_scholar");
        assert_eq!(value["paper_count"], 548);
        assert_eq!(value["citation_count"], 50_000);
        assert_eq!(value["h_index"], 100);
        assert_eq!(value["warnings"][0]["code"], "orcid_link_not_established");
        assert_eq!(value["_meta"]["next_commands"], serde_json::json!([]));
        assert_eq!(
            value["affiliations"][0]["evidence"]["source"],
            "semantic_scholar"
        );
        assert_eq!(value["affiliations"][0]["temporal"]["kind"], "observed_at");
        let json = value.to_string();
        assert!(!json.contains("external_ids"));
        assert!(!json.contains("private-sentinel"));
    }
}
