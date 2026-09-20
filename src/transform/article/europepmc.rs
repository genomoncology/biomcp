use biodata::{PublicationIdentifier, ScientificPublication};

use crate::entities::article::{Article, ArticleAuthorCompleteness, ArticleSource};
use crate::sources::europepmc::EuropePmcDetail;

use super::anchors::clean_title;
use super::federation::{
    parse_open_access_string, publication_type_from_string, retraction_status_from_string,
    split_author_string,
};

pub fn from_europepmc_detail(detail: &EuropePmcDetail) -> Article {
    match detail {
        EuropePmcDetail::Legacy { requested, result } => {
            let mut article = super::federation::retained_from_europepmc_result(result);
            set_requested_identifier(&mut article, requested);
            article
        }
        EuropePmcDetail::Adopted(response) => adopted_article(response),
    }
}

pub fn merge_europepmc_detail_metadata(article: &mut Article, detail: &EuropePmcDetail) {
    match detail {
        EuropePmcDetail::Legacy { result, .. } => {
            super::federation::retained_merge_europepmc_metadata(article, result);
        }
        EuropePmcDetail::Adopted(response) => merge_adopted(article, response),
    }
}

fn adopted_article(response: &biodata::EuropePmcDetailResponse) -> Article {
    let value = response.value();
    let record = response.provider_record();
    let authors = value
        .authorships()
        .first()
        .and_then(|assertion| assertion.raw())
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
        pmid: identifier(value, "pmid"),
        pmcid: identifier(value, "pmcid"),
        doi: identifier(value, "doi"),
        title: clean_title(value.title().raw().unwrap_or_default()),
        author_count: authors.len(),
        authors,
        author_completeness,
        author_source: ArticleSource::EuropePmc,
        journal: value
            .journals()
            .first()
            .map(|assertion| assertion.raw().trim().to_string())
            .filter(|text| !text.is_empty()),
        date: publication_date(value),
        citation_count: record.and_then(|row| row.cited_by_count()),
        publication_type: publication_type_from_string(record.and_then(|row| row.pub_type())),
        open_access: record
            .and_then(|row| row.is_open_access())
            .and_then(parse_open_access_string),
        abstract_text: None,
        full_text_path: None,
        full_text_note: None,
        full_text_source: None,
        full_text_manifest: None,
        full_text_coverage: None,
        not_included: None,
        europepmc_license: None,
        europepmc_retracted: retraction_status_from_string(record.and_then(|row| row.pub_type())),
        annotations: None,
        indexing: None,
        semantic_scholar: None,
        pubtator_fallback: false,
    }
}

fn merge_adopted(article: &mut Article, response: &biodata::EuropePmcDetailResponse) {
    let value = response.value();
    if article.doi.is_none() {
        article.doi = identifier(value, "doi");
    }
    if article.pmcid.is_none() {
        article.pmcid = identifier(value, "pmcid");
    }
    if article.journal.is_none() {
        article.journal = value
            .journals()
            .first()
            .map(|assertion| assertion.raw().trim().to_string())
            .filter(|text| !text.is_empty());
    }
    if article.date.is_none() {
        article.date = publication_date(value);
    }
    if let Some(record) = response.provider_record() {
        article.citation_count = record.cited_by_count();
        article.publication_type = publication_type_from_string(record.pub_type());
        article.open_access = record.is_open_access().and_then(parse_open_access_string);
        article.europepmc_retracted = retraction_status_from_string(record.pub_type());
    }
}

fn set_requested_identifier(article: &mut Article, requested: &PublicationIdentifier) {
    match requested {
        PublicationIdentifier::Pmid(value) => article.pmid = Some(value.as_str().to_string()),
        PublicationIdentifier::Pmcid(value) => article.pmcid = Some(value.as_str().to_string()),
        PublicationIdentifier::Doi(value) => article.doi = Some(value.as_str().to_string()),
    }
}

fn identifier(value: &ScientificPublication, authority: &str) -> Option<String> {
    value
        .identifiers()
        .iter()
        .find_map(|identifier| match (authority, identifier) {
            ("pmid", PublicationIdentifier::Pmid(value)) => Some(value.as_str().to_string()),
            ("pmcid", PublicationIdentifier::Pmcid(value)) => Some(value.as_str().to_string()),
            ("doi", PublicationIdentifier::Doi(value)) => Some(value.as_str().to_string()),
            _ => None,
        })
}

fn publication_date(value: &ScientificPublication) -> Option<String> {
    for label in ["europepmc.first_publication_date", "europepmc.pub_year"] {
        if let Some(raw) = value
            .named_dates()
            .iter()
            .find(|date| date.label().as_str() == label)
            .map(|date| date.raw())
        {
            return Some(raw.get(0..10).unwrap_or(raw).to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detail(pub_type: &str) -> EuropePmcDetail {
        let bytes = serde_json::to_vec(&serde_json::json!({
            "version":"6.9", "hitCount":1,
            "request":{"queryString":"EXT_ID:7 AND SRC:MED", "internalQuery":"x",
                "resultType":"LITE", "cursorMark":"*", "pageSize":1,
                "sort":"", "synonym":false},
            "resultList":{"result":[{"id":"7", "source":"MED", "pmid":"7",
                "title":"title", "pubType":pub_type}]}
        }))
        .unwrap();
        crate::sources::europepmc::parse_publication_detail(
            PublicationIdentifier::Pmid(biodata::Pmid::new("7").unwrap()),
            &bytes,
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn adopted_multitype_uses_first_type_and_retraction_across_from_and_merge() {
        let multitype = detail("Retracted Publication; Journal Article");
        let projected = from_europepmc_detail(&multitype);
        assert_eq!(
            projected.publication_type.as_deref(),
            Some("Retracted Publication")
        );
        assert_eq!(projected.europepmc_retracted, Some(true));

        let mut merged = from_europepmc_detail(&detail("Review"));
        merge_europepmc_detail_metadata(&mut merged, &multitype);
        assert_eq!(
            merged.publication_type.as_deref(),
            Some("Retracted Publication")
        );
        assert_eq!(merged.europepmc_retracted, Some(true));
    }

    #[test]
    fn adopted_blank_type_is_absent_across_from_and_merge() {
        let blank = detail("   ");
        let projected = from_europepmc_detail(&blank);
        assert_eq!(projected.publication_type, None);
        assert_eq!(projected.europepmc_retracted, None);

        let mut merged = from_europepmc_detail(&detail("Review"));
        merge_europepmc_detail_metadata(&mut merged, &blank);
        assert_eq!(merged.publication_type, None);
        assert_eq!(merged.europepmc_retracted, None);
    }
}
