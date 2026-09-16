use crate::entities::article::{Article, ArticleAuthorCompleteness, ArticleSource};
use crate::sources::pubtator::{PubTatorDetail, PubTatorDocument};

use super::anchors::truncate_abstract;

struct PubTatorArticleData<'a> {
    pmid: Option<String>,
    pmcid: Option<&'a str>,
    title: Option<&'a str>,
    abstract_text: Option<&'a str>,
    authors: &'a [String],
    author_completeness: ArticleAuthorCompleteness,
    journal: Option<&'a str>,
    date: Option<&'a str>,
}

pub fn from_pubtator_detail(detail: &PubTatorDetail) -> Article {
    match detail {
        PubTatorDetail::Adopted(response) => {
            let record = response.provider_record();
            let authors = record
                .as_ref()
                .and_then(|value| value.authors())
                .unwrap_or_default();
            assemble_pubtator_article(PubTatorArticleData {
                pmid: response
                    .value()
                    .identifiers()
                    .iter()
                    .find_map(|identifier| {
                        if let biodata::PublicationIdentifier::Pmid(pmid) = identifier {
                            Some(pmid.as_str().to_string())
                        } else {
                            None
                        }
                    }),
                pmcid: None,
                title: response.value().title().raw(),
                abstract_text: response.value().abstract_text().raw(),
                authors,
                author_completeness: if authors.is_empty() {
                    ArticleAuthorCompleteness::Unavailable
                } else {
                    ArticleAuthorCompleteness::SourceLimited
                },
                journal: record.as_ref().and_then(|value| value.journal()),
                date: record.as_ref().and_then(|value| value.date()),
            })
        }
        PubTatorDetail::Legacy {
            requested_pmid,
            document,
        } => from_legacy_pubtator_document(Some(requested_pmid.as_str().to_string()), document),
    }
}

pub fn from_pubtator_document(doc: &PubTatorDocument) -> Article {
    let pmid = doc.pmid.map(|value| value.to_string());
    from_legacy_pubtator_document(pmid, doc)
}

fn from_legacy_pubtator_document(
    requested_pmid: Option<String>,
    doc: &PubTatorDocument,
) -> Article {
    let title = legacy_passage_text(doc, "title");
    let abstract_text = legacy_passage_text(doc, "abstract");
    assemble_pubtator_article(PubTatorArticleData {
        pmid: requested_pmid,
        pmcid: doc.pmcid.as_deref(),
        title,
        abstract_text,
        authors: &doc.authors,
        author_completeness: if doc.authors.is_empty() {
            ArticleAuthorCompleteness::Unavailable
        } else {
            ArticleAuthorCompleteness::Complete
        },
        journal: doc.journal.as_deref(),
        date: doc.date.as_deref(),
    })
}

fn legacy_passage_text<'a>(doc: &'a PubTatorDocument, wanted: &str) -> Option<&'a str> {
    doc.passages.iter().find_map(|passage| {
        let kind = passage.infons.as_ref()?.kind.as_deref()?;
        let text = passage.text.as_deref()?.trim();
        (kind == wanted && !text.is_empty()).then_some(text)
    })
}

fn assemble_pubtator_article(data: PubTatorArticleData<'_>) -> Article {
    let authors = data.authors.to_vec();
    Article {
        section_outcomes: crate::entities::section_outcome::SectionOutcomes::with_keys(
            crate::entities::article::ARTICLE_OUTCOME_KEYS,
        ),
        pmid: data.pmid,
        pmcid: data.pmcid.map(str::to_string),
        doi: None,
        title: data.title.unwrap_or_default().trim().to_string(),
        author_count: authors.len(),
        authors,
        author_completeness: data.author_completeness,
        author_source: ArticleSource::PubTator,
        journal: data
            .journal
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        date: data
            .date
            .and_then(|value| value.get(0..10))
            .map(str::to_string),
        citation_count: None,
        publication_type: None,
        open_access: None,
        abstract_text: data
            .abstract_text
            .map(truncate_abstract)
            .filter(|value| !value.is_empty()),
        full_text_path: None,
        full_text_note: None,
        full_text_source: None,
        full_text_manifest: None,
        full_text_coverage: None,
        not_included: None,
        europepmc_license: None,
        europepmc_retracted: None,
        annotations: None,
        indexing: None,
        semantic_scholar: None,
        pubtator_fallback: false,
    }
}
