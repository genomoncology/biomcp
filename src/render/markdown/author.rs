use crate::entities::author::{AuthorDetail, AuthorIdentity, AuthorSearchResponse, ProviderStatus};
use std::fmt::Write as _;

pub fn author_search_markdown(response: &AuthorSearchResponse) -> String {
    let mut out = format!(
        "# Author search: {}\n\nSource: Semantic Scholar\n\nIdentity: exact provider\n",
        response.query.name
    );
    for provider in &response.providers {
        let _ = writeln!(out, "\nStatus: {}", status(provider.status));
        if let Some(degradation) = &provider.degradation {
            let _ = writeln!(out, "Degradation: {}", degradation.message);
        }
        for result in &provider.results {
            let AuthorIdentity::ExactProvider { id } = &result.identity;
            let _ = writeln!(
                out,
                "\n## {}\n\n- ID: `{id}`\n- ORCID link: not established by BioMCP in this release.",
                result.display_name
            );
        }
    }
    out
}
pub fn author_detail_markdown(author: &AuthorDetail) -> String {
    let AuthorIdentity::ExactProvider { id } = &author.identity;
    let mut out = format!(
        "# {}\n\nSource: Semantic Scholar\n\nIdentity: exact provider\n\n- ID: `{id}`\n- Status: available\n- ORCID link: not established by BioMCP in this release.\n",
        author.display_name
    );
    if !author.affiliations.is_empty() {
        out.push_str("\n## Affiliations\n");
        for affiliation in &author.affiliations {
            let _ = writeln!(out, "- {}", affiliation.value);
        }
    }
    if !author.conflicts.is_empty() {
        out.push_str("\n## Conflicts\n");
        for conflict in &author.conflicts {
            let _ = writeln!(out, "- {}: {}", conflict.field, conflict.values.join(", "));
        }
    }
    out
}
fn status(value: ProviderStatus) -> &'static str {
    match value {
        ProviderStatus::Available => "available",
        ProviderStatus::Degraded => "degraded",
        ProviderStatus::Unavailable => "unavailable",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::author::{
        AuthorEvidenceUrl, AuthorMeta, AuthorSourceStatus, AuthorWarning, ProviderAuthorId,
        ProviderAuthorRecord,
    };

    #[test]
    fn detail_markdown_keeps_provider_identity_and_uncertainty_visible() {
        let id: ProviderAuthorId = "semanticscholar:1716151".parse().unwrap();
        let output = author_detail_markdown(&AuthorDetail {
            identity: AuthorIdentity::ExactProvider { id: id.clone() },
            display_name: "A. Butte".into(),
            provider_records: vec![ProviderAuthorRecord {
                id,
                source: "semantic_scholar",
                status: ProviderStatus::Available,
            }],
            affiliations: vec![],
            paper_count: None,
            citation_count: None,
            h_index: None,
            conflicts: vec![],
            warnings: vec![AuthorWarning::unresolved_orcid()],
            _meta: AuthorMeta {
                source_status: vec![AuthorSourceStatus {
                    source: "semantic_scholar",
                    status: ProviderStatus::Available,
                }],
                evidence_urls: vec![AuthorEvidenceUrl {
                    source: "semantic_scholar",
                    url: "https://www.semanticscholar.org/author/1716151".into(),
                }],
                next_commands: vec![],
            },
        });
        for expected in [
            "Source: Semantic Scholar",
            "Identity: exact provider",
            "semanticscholar:1716151",
            "ORCID link: not established by BioMCP in this release",
        ] {
            assert!(output.contains(expected));
        }
    }
}
