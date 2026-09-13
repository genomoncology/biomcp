//! Article CLI citation-evidence markdown tests.
#[test]
fn citation_evidence_markdown_pins_the_unresolved_and_unlinked_states() {
    use crate::entities::article::ArticleRelatedPaper;
    use crate::entities::article::graph::citation_evidence::{
        ArticleCitationEvidenceResult, CitationEvidenceLocator, CitationEvidenceMeta,
        CitationEvidenceStatus,
    };
    use crate::render::markdown::article_citation_evidence_markdown;

    fn paper(pmid: &str) -> ArticleRelatedPaper {
        ArticleRelatedPaper {
            paper_id: None,
            pmid: Some(pmid.to_string()),
            doi: None,
            arxiv_id: None,
            title: "T".into(),
            journal: None,
            year: None,
        }
    }

    fn result(
        pmid: &str,
        status: CitationEvidenceStatus,
        message: &str,
        pmcid: &str,
    ) -> ArticleCitationEvidenceResult {
        ArticleCitationEvidenceResult {
            citing: paper(pmid),
            cited: paper("0"),
            status,
            message: message.to_string(),
            source: Some("europe_pmc_jats".into()),
            provider_contexts: vec![],
            passages: vec![],
            fulltext_locator: Some(CitationEvidenceLocator {
                pmcid: pmcid.to_string(),
                evidence_url: format!(
                    "https://www.ebi.ac.uk/europepmc/webservices/rest/{pmcid}/fullTextXML"
                ),
            }),
            _meta: CitationEvidenceMeta {
                source_status: vec![],
                evidence_urls: vec![],
                next_commands: vec![],
            },
        }
    }
    let unresolved = result(
        "40001003",
        CitationEvidenceStatus::ReferenceUnresolved,
        "Structured full text was available, but the cited reference could not be resolved exactly.",
        "PMC12923960",
    );
    let expected_unresolved = "# Citation evidence\n\n\
Citing: `PMID 40001003`\n\
Cited: `PMID 0`\n\
Status: Structured full text was available, but the cited reference could not be resolved exactly.\n\
\nFull text: `https://www.ebi.ac.uk/europepmc/webservices/rest/PMC12923960/fullTextXML`\n";
    assert_eq!(
        article_citation_evidence_markdown(&unresolved).unwrap(),
        expected_unresolved
    );

    let unlinked = result(
        "40001004",
        CitationEvidenceStatus::CitationMarkerUnlinked,
        "The cited reference was resolved, but no unambiguous in-text citation marker linked to it.",
        "PMC12923961",
    );
    let expected_unlinked = "# Citation evidence\n\n\
Citing: `PMID 40001004`\n\
Cited: `PMID 0`\n\
Status: The cited reference was resolved, but no unambiguous in-text citation marker linked to it.\n\
\nFull text: `https://www.ebi.ac.uk/europepmc/webservices/rest/PMC12923961/fullTextXML`\n";
    assert_eq!(
        article_citation_evidence_markdown(&unlinked).unwrap(),
        expected_unlinked
    );
}
