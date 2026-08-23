//! Transform adapters for article data from upstream API sources into CLI-facing entity models.

mod anchors;
mod annotations;
mod federation;
mod html;
mod jats;
mod pdf;

#[allow(unused_imports)]
pub use self::anchors::truncate_abstract;
pub use self::anchors::{
    article_search_abstract_snippet, article_search_fallback_title, clean_abstract, clean_title,
    normalize_article_search_text,
};
pub use self::annotations::extract_annotations;
pub use self::federation::{
    from_europepmc_result, from_europepmc_search_result, from_pubmed_esummary_entry,
    from_pubtator_document, from_pubtator_search_result, merge_europepmc_metadata,
};
pub(crate) use self::html::{classify_html_document, extract_pmc_supplement_links};
pub(crate) use self::jats::{classify_jats_document, extract_jats_supplement_links};
pub use self::pdf::extract_text_from_pdf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArticleSupplementLink {
    pub href: String,
    pub filename: String,
    pub label: Option<String>,
    pub caption: Option<String>,
    pub media_type: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArticleDocumentCoverage {
    FullText,
    AbstractOnly,
    MetadataOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClassifiedArticleDocument {
    pub coverage: ArticleDocumentCoverage,
    pub markdown: Option<String>,
    pub abstract_text: Option<String>,
    pub quality: crate::entities::article::ArticleFulltextQuality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(crate) enum ArticleDocumentUnusable {
    #[error("article document was malformed")]
    Malformed,
    #[error("article document was unsupported")]
    Unsupported,
    #[error("article document conversion failed")]
    Conversion,
}

fn collapse_whitespace(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut last_was_space = false;

    for ch in value.chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                out.push(' ');
                last_was_space = true;
            }
        } else {
            out.push(ch);
            last_was_space = false;
        }
    }

    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use crate::entities::article::{Article, ArticleAnnotations, ArticleSearchResult};
    use crate::error::BioMcpError;
    use crate::sources::europepmc::EuropePmcResult;
    use crate::sources::pubmed::ESummaryEntry;
    use crate::sources::pubtator::{PubTatorDocument, PubTatorSearchResult};
    use crate::transform::article::{ArticleDocumentUnusable, ClassifiedArticleDocument};

    #[test]
    fn root_module_reexports_stable_article_transform_api() {
        let _ = crate::transform::article::clean_title as fn(&str) -> String;
        let _ = crate::transform::article::clean_abstract as fn(&str) -> String;
        let _ = crate::transform::article::normalize_article_search_text as fn(&str) -> String;
        let _ = crate::transform::article::article_search_fallback_title as fn(&str) -> String;
        let _ = crate::transform::article::truncate_abstract as fn(&str) -> String;
        let _ = crate::transform::article::article_search_abstract_snippet
            as fn(&str) -> Option<String>;
        let _ =
            crate::transform::article::from_pubtator_document as fn(&PubTatorDocument) -> Article;
        let _ = crate::transform::article::from_europepmc_result as fn(&EuropePmcResult) -> Article;
        let _ = crate::transform::article::merge_europepmc_metadata
            as fn(&mut Article, &EuropePmcResult);
        let _ = crate::transform::article::from_europepmc_search_result
            as fn(&EuropePmcResult) -> Option<ArticleSearchResult>;
        let _ = crate::transform::article::from_pubtator_search_result
            as fn(&PubTatorSearchResult) -> Option<ArticleSearchResult>;
        let _ = crate::transform::article::from_pubmed_esummary_entry
            as fn(&ESummaryEntry) -> Option<ArticleSearchResult>;
        let _ = crate::transform::article::extract_annotations
            as fn(&PubTatorDocument) -> Option<ArticleAnnotations>;
        let _ = crate::transform::article::classify_jats_document
            as fn(&str) -> Result<ClassifiedArticleDocument, ArticleDocumentUnusable>;
        let _ = crate::transform::article::classify_html_document
            as fn(&str, &str) -> Result<ClassifiedArticleDocument, ArticleDocumentUnusable>;
        let _ = crate::transform::article::extract_text_from_pdf
            as fn(&[u8], usize) -> Result<String, BioMcpError>;
    }
}
