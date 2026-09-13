use crate::entities::author::{
    ArticleAuthorsResult, AuthorDetail, AuthorIdProvider, AuthorIdentity, AuthorPaperFull,
    AuthorPapersFullResult, AuthorPapersResult, AuthorSearchResponse, ProviderStatus,
};
use std::fmt::Write as _;

pub fn author_papers_markdown(response: &AuthorPapersResult) -> String {
    let AuthorIdentity::ExactProvider { id } = &response.author;
    if id.provider == AuthorIdProvider::Orcid {
        return orcid_papers_markdown(id, response);
    }
    let mut out = format!("# Papers for `{id}`\n\n");
    for paper in &response.papers {
        let identifier = paper
            .pmid
            .as_deref()
            .or(paper.doi.as_deref())
            .or(paper.arxiv_id.as_deref())
            .or(paper.paper_id.as_deref())
            .unwrap_or("unknown");
        let _ = writeln!(out, "## {}\n\n- ID: `{identifier}`", paper.title);
        if let Some(journal) = &paper.journal {
            let _ = writeln!(out, "- Journal: {journal}");
        }
        if let Some(year) = paper.year {
            let _ = writeln!(out, "- Year: {year}");
        }
        out.push('\n');
    }
    out
}

fn provider_inline(value: &str) -> String {
    crate::render::human::sanitize_provider_inline(value)
}

fn orcid_papers_markdown(
    id: &crate::entities::author::ProviderAuthorId,
    response: &AuthorPapersResult,
) -> String {
    let pagination = &response.pagination;
    let has_more = pagination.next.is_some();
    let mut out = format!(
        "# Papers for {}\n\nSource: ORCID\n\nIdentity: exact provider\n\nStatus: available\nTotal: {}; offset: {}; returned: {}; has more: {}; truncated: {}\n",
        code_span(&safe_inline(&id.to_string())),
        pagination.total.unwrap_or(0),
        pagination.offset,
        response.papers.len(),
        has_more,
        pagination.truncated.unwrap_or(false),
    );
    for (index, paper) in response.papers.iter().enumerate() {
        let _ = write!(out, "\n## Work {}\n\n", index + 1);
        let _ = writeln!(out, "- Title: {}", provider_inline(&paper.title));
        if let Some(work_id) = &paper.work_id {
            let _ = writeln!(out, "- Work ID: {}", code_span(&safe_inline(work_id)));
        }
        if let Some(journal) = &paper.journal {
            let _ = writeln!(out, "- Journal: {}", provider_inline(journal));
        }
        if let Some(year) = paper.year {
            let _ = writeln!(out, "- Year: {year}");
        }
        if let Some(pmid) = &paper.pmid {
            let _ = writeln!(out, "- PMID: {}", code_span(&safe_inline(pmid)));
        }
        if let Some(pmcid) = &paper.pmcid {
            let _ = writeln!(out, "- PMCID: {}", code_span(&safe_inline(pmcid)));
        }
        if let Some(doi) = &paper.doi {
            let _ = writeln!(out, "- DOI: {}", code_span(&safe_inline(doi)));
        }
        if let Some(arxiv_id) = &paper.arxiv_id {
            let _ = writeln!(out, "- arXiv ID: {}", code_span(&safe_inline(arxiv_id)));
        }
        for identifier in &paper.identifiers {
            let _ = writeln!(
                out,
                "- Identifier: {}:{}",
                provider_inline(&identifier.kind),
                provider_inline(&identifier.value)
            );
        }
    }
    let commands: Vec<String> = response
        ._meta
        .next_commands
        .iter()
        .map(|command| command.trim().to_string())
        .filter(|command| !command.is_empty())
        .collect();
    if !commands.is_empty() {
        out.push_str("\nSee also:");
        for command in commands {
            let _ = write!(out, "\n  {command}");
        }
        out.push('\n');
    }
    out
}

fn safe_inline(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in crate::render::human::sanitize_inline(value).chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '|' => escaped.push_str("\\|"),
            '<' => escaped.push_str("\\<"),
            '>' => escaped.push_str("\\>"),
            '[' => escaped.push_str("\\["),
            ']' => escaped.push_str("\\]"),
            '`' => escaped.push_str("\\`"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn code_span(value: &str) -> String {
    super::support::markdown_code_span(value)
}

fn code_span_slot(value: Option<&str>) -> String {
    match value {
        None => "unknown".to_string(),
        Some(value) => code_span(&safe_inline(value)),
    }
}

fn number_slot<T: std::fmt::Display>(value: Option<T>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn inline_slot(value: Option<&str>) -> String {
    value.map(safe_inline).unwrap_or_else(|| "unknown".into())
}

fn list_slot(value: Option<&Vec<String>>) -> String {
    match value {
        None => "unknown".into(),
        Some(items) if items.is_empty() => "none".into(),
        Some(items) => items
            .iter()
            .map(|item| safe_inline(item))
            .collect::<Vec<_>>()
            .join(", "),
    }
}

pub fn author_papers_full_markdown(response: &AuthorPapersFullResult) -> String {
    let AuthorIdentity::ExactProvider { id } = &response.author;
    let mut out = format!("# Papers for {}\n\n", code_span(&id.to_string()));
    for (index, paper) in response.papers.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(&paper_block(paper));
    }
    if let Some(next) = response.pagination.next {
        let _ = writeln!(
            out,
            "\nNext: {}",
            code_span(&format!(
                "biomcp author papers {id} --full --limit {} --offset {next}",
                response.pagination.limit
            ))
        );
    }
    out
}

fn paper_block(paper: &AuthorPaperFull) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## {}\n", safe_inline(&paper.title));
    let _ = writeln!(
        out,
        "- Paper ID: {}",
        code_span(&safe_inline(&paper.paper_id))
    );
    let _ = writeln!(out, "- Corpus ID: {}", number_slot(paper.corpus_id));
    let _ = writeln!(out, "- PMID: {}", code_span_slot(paper.pmid.as_deref()));
    let _ = writeln!(out, "- PMCID: {}", code_span_slot(paper.pmcid.as_deref()));
    let _ = writeln!(out, "- DOI: {}", code_span_slot(paper.doi.as_deref()));
    let _ = writeln!(
        out,
        "- arXiv ID: {}",
        code_span_slot(paper.arxiv_id.as_deref())
    );
    let _ = writeln!(out, "- Journal: {}", inline_slot(paper.journal.as_deref()));
    let _ = writeln!(out, "- Year: {}", number_slot(paper.year));
    let _ = writeln!(
        out,
        "- Publication date: {}",
        code_span_slot(paper.publication_date.as_deref())
    );
    let _ = writeln!(out, "- Citations: {}", number_slot(paper.citation_count));
    let _ = writeln!(out, "- References: {}", number_slot(paper.reference_count));
    let _ = writeln!(
        out,
        "- Influential citations: {}",
        number_slot(paper.influential_citation_count)
    );
    let _ = writeln!(
        out,
        "- Open access: {}",
        paper
            .is_open_access
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".into())
    );
    let pdf = paper.open_access_pdf.as_ref();
    let pdf_slot = |value: Option<&str>| match (pdf, value) {
        (Some(_), Some(value)) => code_span(&safe_inline(value)),
        _ => "unknown".to_string(),
    };
    let _ = writeln!(
        out,
        "- Open-access PDF URL: {}",
        pdf_slot(pdf.and_then(|pdf| pdf.url.as_deref()))
    );
    let _ = writeln!(
        out,
        "- Open-access PDF status: {}",
        pdf_slot(pdf.and_then(|pdf| pdf.status.as_deref()))
    );
    let _ = writeln!(
        out,
        "- Open-access PDF license: {}",
        pdf_slot(pdf.and_then(|pdf| pdf.license.as_deref()))
    );
    let _ = writeln!(
        out,
        "- Fields of study: {}",
        list_slot(paper.fields_of_study.as_ref())
    );
    let _ = writeln!(
        out,
        "- Publication types: {}",
        list_slot(paper.publication_types.as_ref())
    );
    out.push_str("\n### Authors\n\n");
    match paper.authors.as_ref() {
        None => out.push_str("- unknown\n"),
        Some(byline) if byline.is_empty() => out.push_str("- none\n"),
        Some(byline) => {
            for (index, author) in byline.iter().enumerate() {
                let name = author
                    .display_name
                    .as_deref()
                    .map(safe_inline)
                    .unwrap_or_else(|| "unknown".into());
                let id = match &author.identity {
                    Some(AuthorIdentity::ExactProvider { id }) => code_span(&id.to_string()),
                    None => "unknown".to_string(),
                };
                let _ = writeln!(out, "{}. {} ({})", index + 1, name, id);
            }
        }
    }
    out.push_str("\n### Abstract\n\n");
    let _ = writeln!(out, "{}", inline_slot(paper.abstract_text.as_deref()));
    out
}

pub fn article_authors_markdown(response: &ArticleAuthorsResult) -> String {
    let mut out = format!("# Authors for {}\n\n", response.article.title);
    for author in &response.authors {
        let AuthorIdentity::ExactProvider { id } = &author.identity;
        let _ = writeln!(out, "## {}\n\n- ID: `{id}`", author.display_name);
        if !author.affiliations.is_empty() {
            let values = author
                .affiliations
                .iter()
                .map(|assertion| assertion.value.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            let _ = writeln!(out, "- Affiliations: {values}");
        }
        out.push('\n');
    }
    out
}

pub fn author_search_markdown(response: &AuthorSearchResponse) -> String {
    let mut out = format!(
        "# Author search: {}\n\nSource: Semantic Scholar\n\nIdentity: exact provider\n",
        response.query.name
    );
    for provider in &response.providers {
        let _ = writeln!(out, "\nStatus: {}", status(provider.status));
        let pagination = &provider.pagination;
        let _ = writeln!(
            out,
            "Total: {}; offset: {}; returned: {}; has more: {}",
            pagination
                .total
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".into()),
            pagination.offset,
            provider.results.len(),
            pagination.next.is_some()
        );
        if let Some(degradation) = &provider.degradation {
            let _ = writeln!(out, "Degradation: {}", degradation.message);
        }
        for result in &provider.results {
            let AuthorIdentity::ExactProvider { id } = &result.identity;
            let _ = writeln!(
                out,
                "\n## {}\n\n- ID: `{id}`\n- Affiliation: {}\n- Papers: {}\n- Citations: {}\n- h-index: {}\n- ORCID link: not established by BioMCP in this release.",
                result.display_name,
                result
                    .affiliations
                    .first()
                    .map(|value| truncate_affiliation(&value.value))
                    .unwrap_or_else(|| "unknown".into()),
                metric(result.paper_count),
                metric(result.citation_count),
                metric(result.h_index)
            );
        }
        if let Some(next) = pagination.next {
            let _ = writeln!(
                out,
                "\nNext: `biomcp search author --query {} --limit {} --offset {next}`",
                crate::render::markdown::shell_quote_arg(&response.query.name),
                pagination.limit
            );
        }
    }
    out
}

fn metric(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn truncate_affiliation(value: &str) -> String {
    const MAX: usize = 120;
    if value.len() <= MAX {
        return value.to_string();
    }
    let mut end = MAX - '…'.len_utf8();
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &value[..end])
}
pub fn author_detail_markdown(author: &AuthorDetail) -> String {
    let AuthorIdentity::ExactProvider { id } = &author.identity;
    if id.provider == AuthorIdProvider::Orcid {
        let mut out = format!(
            "# {}\n\nSource: ORCID\n\nIdentity: exact provider\n\n- ID: {}\n- Status: available\n",
            provider_inline(&author.display_name),
            code_span(&safe_inline(&id.to_string())),
        );
        let commands: Vec<String> = author
            ._meta
            .next_commands
            .iter()
            .map(|command| command.trim().to_string())
            .filter(|command| !command.is_empty())
            .collect();
        if !commands.is_empty() {
            out.push_str("\nSee also:");
            for command in commands {
                let _ = write!(out, "\n  {command}");
            }
            out.push('\n');
        }
        return out;
    }
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
    let related_block = super::format_related_block(author._meta.next_commands.clone());
    if !related_block.is_empty() {
        let _ = write!(out, "\n{related_block}\n");
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
    fn detail_markdown_and_json_present_the_same_follow_up_commands() {
        let id: ProviderAuthorId = "semanticscholar:1716151".parse().unwrap();
        let author = AuthorDetail {
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
                next_commands: vec!["biomcp author papers semanticscholar:1716151".to_string()],
            },
        };

        let output = author_detail_markdown(&author);
        for expected in [
            "Source: Semantic Scholar",
            "Identity: exact provider",
            "semanticscholar:1716151",
            "ORCID link: not established by BioMCP in this release",
        ] {
            assert!(output.contains(expected));
        }

        let json = serde_json::to_value(&author).unwrap();
        let json_commands: std::collections::BTreeSet<_> = json["_meta"]["next_commands"]
            .as_array()
            .unwrap()
            .iter()
            .map(|command| command.as_str().unwrap().to_string())
            .collect();
        let markdown_commands: std::collections::BTreeSet<_> = output
            .lines()
            .filter_map(|line| line.strip_prefix("  biomcp "))
            .map(|command| {
                let command = command.split("   - ").next().unwrap();
                format!("biomcp {command}")
            })
            .collect();

        assert_eq!(markdown_commands, json_commands);
    }

    #[test]
    fn affiliation_preview_is_bounded_without_splitting_utf8() {
        let shortened = truncate_affiliation(&"é".repeat(100));
        assert!(shortened.len() <= 120);
        assert!(shortened.ends_with('…'));
        assert!(shortened.is_char_boundary(shortened.len()));
        assert_eq!(metric(None), "unknown");
        assert_eq!(metric(Some(0)), "0");
    }
}

#[cfg(test)]
mod full_page_tests {
    use super::*;
    use crate::entities::author::{
        AuthorEvidenceUrl, AuthorMeta, AuthorPaperFull, AuthorPaperFullAuthor,
        AuthorPaperOpenAccessPdf, AuthorPapersFullResult, AuthorPapersPagination,
        AuthorSourceStatus, ProviderStatus,
    };

    fn full_result(papers: Vec<AuthorPaperFull>, next: Option<u64>) -> AuthorPapersFullResult {
        AuthorPapersFullResult {
            author: AuthorIdentity::ExactProvider {
                id: "semanticscholar:1716151".parse().unwrap(),
            },
            papers,
            pagination: AuthorPapersPagination {
                offset: 0,
                limit: 10,
                next,
                total: None,
                truncated: None,
            },
            _meta: AuthorMeta {
                source_status: vec![AuthorSourceStatus {
                    source: "semantic_scholar",
                    status: ProviderStatus::Available,
                }],
                evidence_urls: vec![AuthorEvidenceUrl {
                    source: "semantic_scholar",
                    url: "https://www.semanticscholar.org/paper/paper-identity-1".into(),
                }],
                next_commands: Vec::new(),
            },
        }
    }

    #[test]
    fn rich_markdown_pins_the_exact_template_for_a_complete_paper() {
        let paper = AuthorPaperFull {
            paper_id: "0123456789abcdef0123456789abcdef01234567".into(),
            corpus_id: Some(277710284),
            pmid: Some("40215974".into()),
            pmcid: Some("".into()),
            doi: Some("10.1016/j.fixture.2024.01.001".into()),
            arxiv_id: None,
            title: "A rich author paper fixture".into(),
            abstract_text: Some("Source abstract.".into()),
            journal: Some("Fixture Medicine".into()),
            year: Some(2024),
            publication_date: Some("2024-01-31".into()),
            citation_count: Some(17),
            reference_count: Some(0),
            influential_citation_count: Some(2),
            is_open_access: Some(false),
            open_access_pdf: Some(AuthorPaperOpenAccessPdf {
                url: Some("https://example.invalid/paper.pdf".into()),
                status: Some("HYBRID".into()),
                license: None,
            }),
            fields_of_study: Some(vec!["Medicine".into()]),
            publication_types: Some(vec!["JournalArticle".into()]),
            authors: Some(vec![AuthorPaperFullAuthor {
                identity: Some(AuthorIdentity::ExactProvider {
                    id: "semanticscholar:2059910739".parse().unwrap(),
                }),
                display_name: Some("First Author".into()),
            }]),
        };
        let markdown = author_papers_full_markdown(&full_result(vec![paper], None));
        assert_eq!(
            markdown,
            "# Papers for `semanticscholar:1716151`\n\
             \n\
             ## A rich author paper fixture\n\
             \n\
             - Paper ID: `0123456789abcdef0123456789abcdef01234567`\n\
             - Corpus ID: 277710284\n\
             - PMID: `40215974`\n\
             - PMCID: ``\n\
             - DOI: `10.1016/j.fixture.2024.01.001`\n\
             - arXiv ID: unknown\n\
             - Journal: Fixture Medicine\n\
             - Year: 2024\n\
             - Publication date: `2024-01-31`\n\
             - Citations: 17\n\
             - References: 0\n\
             - Influential citations: 2\n\
             - Open access: false\n\
             - Open-access PDF URL: `https://example.invalid/paper.pdf`\n\
             - Open-access PDF status: `HYBRID`\n\
             - Open-access PDF license: unknown\n\
             - Fields of study: Medicine\n\
             - Publication types: JournalArticle\n\
             \n\
             ### Authors\n\
             \n\
             1. First Author (`semanticscholar:2059910739`)\n\
             \n\
             ### Abstract\n\
             \n\
             Source abstract.\n"
        );
    }

    #[test]
    fn rich_markdown_sentinels_and_two_blocks_with_continuation() {
        let unknowns = AuthorPaperFull {
            paper_id: "opaque".into(),
            corpus_id: None,
            pmid: None,
            pmcid: None,
            doi: None,
            arxiv_id: None,
            title: "All unknown".into(),
            abstract_text: None,
            journal: None,
            year: None,
            publication_date: None,
            citation_count: None,
            reference_count: None,
            influential_citation_count: None,
            is_open_access: None,
            open_access_pdf: None,
            fields_of_study: None,
            publication_types: None,
            authors: None,
        };
        let empties = AuthorPaperFull {
            paper_id: "opaque2".into(),
            title: "All empty".into(),
            abstract_text: Some("".into()),
            journal: Some("".into()),
            fields_of_study: Some(Vec::new()),
            publication_types: Some(Vec::new()),
            authors: Some(Vec::new()),
            open_access_pdf: Some(AuthorPaperOpenAccessPdf {
                url: None,
                status: None,
                license: None,
            }),
            corpus_id: Some(0),
            citation_count: Some(0),
            reference_count: Some(0),
            influential_citation_count: Some(0),
            is_open_access: Some(false),
            ..unknowns.clone()
        };
        let markdown = author_papers_full_markdown(&full_result(vec![unknowns, empties], Some(11)));
        assert!(markdown.contains("- Corpus ID: unknown\n"));
        assert!(markdown.contains("- Open access: unknown\n"));
        assert!(markdown.contains("- Open-access PDF URL: unknown\n"));
        assert!(markdown.contains("- Fields of study: unknown\n"));
        assert!(markdown.contains("- Publication types: unknown\n"));
        assert!(markdown.contains("### Authors\n\n- unknown\n"));
        assert!(markdown.contains("- Publication types: none\n"));
        assert!(markdown.contains("### Authors\n\n- none\n"));
        assert!(markdown.contains("- Corpus ID: 0\n"));
        assert!(markdown.contains("- Open access: false\n"));
        assert_eq!(markdown.matches("\n## ").count(), 2, "two paper blocks");
        assert!(
            markdown.contains("unknown\n\n## All empty"),
            "one blank line separates blocks"
        );
        assert!(markdown.ends_with("\n") && !markdown.ends_with("\n\n"));
        assert!(markdown.ends_with(
            "Next: `biomcp author papers semanticscholar:1716151 --full --limit 10 --offset 11`\n"
        ));
        assert!(markdown.contains("### Abstract\n\nunknown\n"));
        assert!(
            markdown.matches("### Abstract").count() == 2,
            "abstract slot per block"
        );
    }

    #[test]
    fn hostile_provider_values_cannot_inject_markdown_structure() {
        let hostile = AuthorPaperFull {
            paper_id: "A/?#% \n雪".into(),
            title: "Title | with pipes <b> `code`".into(),
            journal: Some("J | rnal\n# Heading".into()),
            abstract_text: Some("[link](https://evil.invalid) <script>x</script>".into()),
            fields_of_study: Some(vec!["A | B".into(), "`` C".into()]),
            authors: Some(vec![AuthorPaperFullAuthor {
                identity: None,
                display_name: Some("Name | `x` #h".into()),
            }]),
            ..crate::entities::author::AuthorPaperFull {
                paper_id: String::new(),
                title: String::new(),
                corpus_id: None,
                pmid: None,
                pmcid: None,
                doi: None,
                arxiv_id: None,
                abstract_text: None,
                journal: None,
                year: None,
                publication_date: None,
                citation_count: None,
                reference_count: None,
                influential_citation_count: None,
                is_open_access: None,
                open_access_pdf: None,
                fields_of_study: None,
                publication_types: None,
                authors: None,
            }
        };
        let markdown = author_papers_full_markdown(&full_result(vec![hostile], None));
        assert!(
            markdown.contains("- Paper ID: `A/?#% 雪`"),
            "control newline coalesces with the adjacent space: {markdown}"
        );
        assert!(!markdown.contains("<script>"));
        assert!(
            !markdown.contains("[link](https"),
            "markdown link syntax is escaped"
        );
        assert!(!markdown.contains("<b>"));
        assert!(!markdown.contains("\n# Heading"));
        assert!(markdown.ends_with('\n'));
    }
}

#[cfg(test)]
mod orcid_render_tests {
    use super::*;
    use crate::entities::author::{
        AuthorEvidenceUrl, AuthorMeta, AuthorPaper, AuthorPaperIdentifier, AuthorPapersPagination,
        AuthorPapersResult, AuthorSourceStatus, ProviderAuthorId, ProviderAuthorRecord,
    };

    fn orcid_detail(display_name: &str) -> AuthorDetail {
        let id: ProviderAuthorId = "orcid:0000-0002-1825-0097".parse().unwrap();
        AuthorDetail {
            identity: AuthorIdentity::ExactProvider { id: id.clone() },
            display_name: display_name.to_string(),
            provider_records: vec![ProviderAuthorRecord {
                id,
                source: "orcid",
                status: ProviderStatus::Available,
            }],
            affiliations: vec![],
            paper_count: None,
            citation_count: None,
            h_index: None,
            conflicts: vec![],
            warnings: vec![],
            _meta: AuthorMeta {
                source_status: vec![AuthorSourceStatus {
                    source: "orcid",
                    status: ProviderStatus::Available,
                }],
                evidence_urls: vec![AuthorEvidenceUrl {
                    source: "orcid",
                    url: "https://orcid.org/0000-0002-1825-0097".into(),
                }],
                next_commands: vec!["biomcp author papers orcid:0000-0002-1825-0097".into()],
            },
        }
    }

    #[test]
    fn orcid_detail_markdown_is_the_frozen_template() {
        let output = author_detail_markdown(&orcid_detail("Josiah Carberry"));
        assert_eq!(
            output,
            "# Josiah Carberry\n\nSource: ORCID\n\nIdentity: exact provider\n\n- ID: `orcid:0000-0002-1825-0097`\n- Status: available\n\nSee also:\n  biomcp author papers orcid:0000-0002-1825-0097\n"
        );
    }

    #[test]
    fn orcid_detail_markdown_sanitizes_the_provider_display_name() {
        let output = author_detail_markdown(&orcid_detail("Josiah |<b>`Car$berry`"));
        assert!(output.starts_with("# Josiah"));
        assert!(!output.contains("<b>"));
        assert!(!output.contains("|<"));
        assert!(output.contains("See also:\n  biomcp author papers orcid:0000-0002-1825-0097\n"));
    }

    fn orcid_papers(
        papers: Vec<AuthorPaper>,
        next: Option<u64>,
        total: u64,
        commands: Vec<String>,
    ) -> AuthorPapersResult {
        let id: ProviderAuthorId = "orcid:0000-0002-1825-0097".parse().unwrap();
        AuthorPapersResult {
            author: AuthorIdentity::ExactProvider { id: id.clone() },
            papers,
            pagination: AuthorPapersPagination {
                offset: 0,
                limit: 1,
                next,
                total: Some(total),
                truncated: Some(false),
            },
            _meta: AuthorMeta {
                source_status: vec![AuthorSourceStatus {
                    source: "orcid",
                    status: ProviderStatus::Available,
                }],
                evidence_urls: vec![AuthorEvidenceUrl {
                    source: "orcid",
                    url: "https://orcid.org/0000-0002-1825-0097/work/42".into(),
                }],
                next_commands: commands,
            },
        }
    }

    #[test]
    fn orcid_papers_markdown_is_the_frozen_nonterminal_template() {
        let paper = AuthorPaper {
            paper_id: None,
            pmid: Some("123".into()),
            doi: Some("10.1/example".into()),
            arxiv_id: None,
            title: "A claimed work".into(),
            journal: Some("A Journal".into()),
            year: Some(2024),
            work_id: Some("orcid:0000-0002-1825-0097/work:42".into()),
            pmcid: Some("PMC456".into()),
            identifiers: vec![
                AuthorPaperIdentifier {
                    kind: "pmid".into(),
                    value: "123".into(),
                },
                AuthorPaperIdentifier {
                    kind: "doi".into(),
                    value: "10.1/example".into(),
                },
                AuthorPaperIdentifier {
                    kind: "pmcid".into(),
                    value: "PMC456".into(),
                },
            ],
        };
        let response = orcid_papers(
            vec![paper],
            Some(1),
            2,
            vec![
                "biomcp get article 123".into(),
                "biomcp author papers orcid:0000-0002-1825-0097 --limit 1 --offset 1".into(),
            ],
        );
        let output = author_papers_markdown(&response);
        assert_eq!(
            output,
            "# Papers for `orcid:0000-0002-1825-0097`\n\nSource: ORCID\n\nIdentity: exact provider\n\nStatus: available\nTotal: 2; offset: 0; returned: 1; has more: true; truncated: false\n\n## Work 1\n\n- Title: A claimed work\n- Work ID: `orcid:0000-0002-1825-0097/work:42`\n- Journal: A Journal\n- Year: 2024\n- PMID: `123`\n- PMCID: `PMC456`\n- DOI: `10.1/example`\n- Identifier: pmid:123\n- Identifier: doi:10&#46;1&#47;example\n- Identifier: pmcid:PMC456\n\nSee also:\n  biomcp get article 123\n  biomcp author papers orcid:0000-0002-1825-0097 --limit 1 --offset 1\n"
        );
    }

    #[test]
    fn orcid_papers_markdown_empty_page_has_no_work_block_or_see_also() {
        let response = orcid_papers(vec![], None, 0, vec![]);
        let output = author_papers_markdown(&response);
        assert_eq!(
            output,
            "# Papers for `orcid:0000-0002-1825-0097`\n\nSource: ORCID\n\nIdentity: exact provider\n\nStatus: available\nTotal: 0; offset: 0; returned: 0; has more: false; truncated: false\n"
        );
    }

    #[test]
    fn orcid_papers_markdown_sanitizes_provider_text() {
        let paper = AuthorPaper {
            paper_id: None,
            pmid: None,
            doi: None,
            arxiv_id: None,
            title: "Hostile |<b>`Title$()".into(),
            journal: Some("J|ournal\nLine".into()),
            year: None,
            work_id: Some("orcid:0000-0002-1825-0097/work:7".into()),
            pmcid: None,
            identifiers: vec![AuthorPaperIdentifier {
                kind: "grant".into(),
                value: "A|B".into(),
            }],
        };
        let response = orcid_papers(vec![paper], None, 1, vec![]);
        let output = author_papers_markdown(&response);
        assert!(!output.contains("<b>"));
        assert!(!output.contains("Hostile |"));
        assert!(output.contains("- Journal: J&#124;ournal Line"));
        assert!(output.contains("- Identifier: grant:A&#124;B"));
    }
}
