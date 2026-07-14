//! Federation mapping regression tests.

use super::*;
use crate::entities::article::ArticleSource;
use crate::sources::europepmc::EuropePmcResult;
use crate::sources::pubmed::ESummaryEntry;
use crate::sources::pubtator::{PubTatorDocument, PubTatorSearchResult};
use chrono::NaiveDate;

#[test]
fn article_sections_maps_egfr_review() {
    let hit: EuropePmcResult = serde_json::from_value(serde_json::json!({
        "id": "39876543",
        "pmid": "39876543",
        "title": "EGFR &lt;i&gt;targeted&lt;/i&gt; therapy in NSCLC",
        "journalTitle": "Cancer Reviews",
        "firstPublicationDate": "2025-03-01",
        "authorString": "A. One, B. Two, C. Three",
        "citedByCount": "24",
        "pubType": "Review Article",
        "isOpenAccess": "Y",
        "abstractText": "EGFR inhibition improves progression-free survival in selected cohorts."
    }))
    .expect("valid Europe PMC hit");

    let article = from_europepmc_result(&hit);
    assert_eq!(article.pmid.as_deref(), Some("39876543"));
    assert!(article.title.contains("EGFR targeted therapy"));
    assert_eq!(article.publication_type.as_deref(), Some("Review"));
    assert_eq!(article.open_access, Some(true));
    assert_eq!(article.authors, vec!["A. One", "B. Two", "C. Three"]);
    assert_eq!(article.author_count, article.authors.len());
    assert_eq!(
        article.author_completeness,
        ArticleAuthorCompleteness::SourceLimited
    );
    assert_eq!(article.author_source, ArticleSource::EuropePmc);
    assert!(
        article
            .abstract_text
            .as_deref()
            .is_some_and(|text| text.contains("EGFR inhibition"))
    );
    assert!(!article.pubtator_fallback);
}

#[test]
fn article_sections_maps_brca1_study() {
    let doc: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "pmid": 22663011,
        "pmcid": "PMC1234567",
        "date": "2024-09-20",
        "journal": "J Clin Oncol",
        "authors": ["Author A", "Author B", "Author C", "Author D", "Author E"],
        "passages": [
            {"infons": {"type": "title"}, "text": "BRCA1 pathogenic variants in breast cancer"},
            {"infons": {"type": "abstract"}, "text": "Study of BRCA1 germline alterations and PARP response."}
        ]
    }))
    .expect("valid PubTator document");

    let article = from_pubtator_document(&doc);
    assert_eq!(article.pmid.as_deref(), Some("22663011"));
    assert_eq!(article.pmcid.as_deref(), Some("PMC1234567"));
    assert!(article.title.contains("BRCA1"));
    assert_eq!(
        article.authors,
        vec!["Author A", "Author B", "Author C", "Author D", "Author E"]
    );
    assert_eq!(article.author_count, article.authors.len());
    assert_eq!(
        article.author_completeness,
        ArticleAuthorCompleteness::Complete
    );
    assert_eq!(article.author_source, ArticleSource::PubTator);
    assert!(!article.pubtator_fallback);
}

#[test]
fn article_authorship_preserves_large_source_lists_and_empty_state() {
    let authors = (1..=28)
        .map(|idx| format!("Author {idx}"))
        .collect::<Vec<_>>();
    let populated_doc: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "authors": authors.clone()
    }))
    .expect("valid PubTator document");
    let populated = from_pubtator_document(&populated_doc);
    assert_eq!(populated.authors, authors);
    assert_eq!(populated.author_count, populated.authors.len());
    assert_eq!(
        populated.author_completeness,
        ArticleAuthorCompleteness::Complete
    );

    let empty_doc: PubTatorDocument =
        serde_json::from_value(serde_json::json!({})).expect("valid PubTator document");
    let empty = from_pubtator_document(&empty_doc);
    assert!(empty.authors.is_empty());
    assert_eq!(empty.author_count, 0);
    assert_eq!(
        empty.author_completeness,
        ArticleAuthorCompleteness::Unavailable
    );
}

#[test]
fn europepmc_authorship_has_no_ten_name_cap_and_reports_empty_state() {
    let names = (1..=18)
        .map(|idx| format!("Author {idx}"))
        .collect::<Vec<_>>();
    let populated_hit: EuropePmcResult = serde_json::from_value(serde_json::json!({
        "id": "1",
        "authorString": names.join(", ")
    }))
    .expect("valid Europe PMC hit");
    let populated = from_europepmc_result(&populated_hit);
    assert_eq!(populated.authors, names);
    assert_eq!(populated.author_count, populated.authors.len());
    assert_eq!(
        populated.author_completeness,
        ArticleAuthorCompleteness::SourceLimited
    );

    let empty_hit: EuropePmcResult =
        serde_json::from_value(serde_json::json!({"id": "2"})).expect("valid Europe PMC hit");
    let empty = from_europepmc_result(&empty_hit);
    assert!(empty.authors.is_empty());
    assert_eq!(empty.author_count, 0);
    assert_eq!(
        empty.author_completeness,
        ArticleAuthorCompleteness::Unavailable
    );
}

#[test]
fn europepmc_metadata_merge_preserves_pubtator_authorship() {
    let doc: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "authors": ["Structured One", "Structured Two"]
    }))
    .expect("valid PubTator document");
    let mut article = from_pubtator_document(&doc);
    let hit: EuropePmcResult = serde_json::from_value(serde_json::json!({
        "id": "1",
        "authorString": "Display One, Display Two"
    }))
    .expect("valid Europe PMC hit");

    merge_europepmc_metadata(&mut article, &hit);

    assert_eq!(article.authors, doc.authors);
    assert_eq!(article.author_count, article.authors.len());
    assert_eq!(
        article.author_completeness,
        ArticleAuthorCompleteness::Complete
    );
    assert_eq!(article.author_source, ArticleSource::PubTator);
}

#[test]
fn publication_type_detection_reads_pub_type_list_for_retractions() {
    let hit: EuropePmcResult = serde_json::from_value(serde_json::json!({
        "id": "1",
        "pmid": "1",
        "title": "Retracted paper",
        "pubTypeList": {
            "pubType": ["Journal Article", "Retracted Publication"]
        }
    }))
    .expect("valid Europe PMC hit");

    let row = from_europepmc_search_result(&hit).expect("search row should map");
    assert_eq!(row.is_retracted, Some(true));
}

#[test]
fn from_europepmc_search_result_carries_first_index_date() {
    let hit: EuropePmcResult = serde_json::from_value(serde_json::json!({
        "id": "1",
        "pmid": "1",
        "title": "Indexed row",
        "firstIndexDate": "2025-01-15"
    }))
    .expect("valid Europe PMC hit");

    let row = from_europepmc_search_result(&hit).expect("search row should map");
    assert_eq!(
        row.first_index_date,
        Some(NaiveDate::from_ymd_opt(2025, 1, 15).expect("valid date"))
    );
}

#[test]
fn from_europepmc_search_result_invalid_first_index_date_is_none() {
    let hit: EuropePmcResult = serde_json::from_value(serde_json::json!({
        "id": "1",
        "pmid": "1",
        "title": "Indexed row",
        "firstIndexDate": "2025-99-15"
    }))
    .expect("valid Europe PMC hit");

    let row = from_europepmc_search_result(&hit).expect("search row should map");
    assert_eq!(row.first_index_date, None);
}

#[test]
fn from_pubtator_search_result_maps_source_and_score() {
    let hit: PubTatorSearchResult = serde_json::from_value(serde_json::json!({
        "_id": "22663011",
        "pmid": 22663011,
        "title": "BRAF in melanoma",
        "journal": "J Clin Oncol",
        "date": "2024-01-20T00:00:00Z",
        "score": 255.9
    }))
    .expect("valid pubtator search row");

    let row = from_pubtator_search_result(&hit).expect("row should map");
    assert_eq!(row.pmid, "22663011");
    assert_eq!(row.source, ArticleSource::PubTator);
    assert_eq!(row.score, Some(255.9));
    assert_eq!(row.citation_count, None);
    assert_eq!(row.is_retracted, None);
}

#[test]
fn parse_sortpubdate_extracts_ymd() {
    assert_eq!(
        parse_sortpubdate("2023/01/15 00:00"),
        Some("2023-01-15".to_string())
    );
}

#[test]
fn parse_pubdate_extracts_full_date() {
    assert_eq!(parse_pubdate("2023 Jan 15"), Some("2023-01-15".to_string()));
}

#[test]
fn parse_pubdate_extracts_year_month() {
    assert_eq!(parse_pubdate("2023 Jan"), Some("2023-01".to_string()));
}

#[test]
fn parse_pubdate_extracts_year() {
    assert_eq!(parse_pubdate("2023 Spring"), Some("2023".to_string()));
}

#[test]
fn from_pubmed_esummary_entry_hydrates_all_fields() {
    let row = from_pubmed_esummary_entry(&ESummaryEntry {
        uid: "12345".into(),
        title: "BRAF &lt;i&gt;targeted&lt;/i&gt; therapy".into(),
        sortpubdate: Some("2023/01/15 00:00".into()),
        pubdate: Some("2023 Jan 15".into()),
        edat: None,
        lr: None,
        fulljournalname: Some("Nature Medicine".into()),
        source: Some("Nat Med".into()),
    })
    .expect("pubmed row should map");

    assert_eq!(row.pmid, "12345");
    assert_eq!(row.title, "BRAF targeted therapy");
    assert_eq!(row.date.as_deref(), Some("2023-01-15"));
    assert_eq!(row.journal.as_deref(), Some("Nature Medicine"));
    assert_eq!(row.source, ArticleSource::PubMed);
    assert_eq!(row.matched_sources, vec![ArticleSource::PubMed]);
    assert_eq!(row.normalized_title, "braf targeted therapy");
    assert_eq!(row.normalized_abstract, "");
    assert_eq!(row.publication_type, None);
}

#[test]
fn from_pubmed_esummary_entry_falls_back_to_source_journal() {
    let row = from_pubmed_esummary_entry(&ESummaryEntry {
        uid: "12345".into(),
        title: "PubMed fallback title".into(),
        sortpubdate: None,
        pubdate: Some("2023 Jan".into()),
        edat: None,
        lr: None,
        fulljournalname: Some("   ".into()),
        source: Some("Nat Med".into()),
    })
    .expect("pubmed row should map");

    assert_eq!(row.date.as_deref(), Some("2023-01"));
    assert_eq!(row.journal.as_deref(), Some("Nat Med"));
}

#[test]
fn from_pubmed_esummary_entry_returns_none_for_blank_title() {
    let row = from_pubmed_esummary_entry(&ESummaryEntry {
        uid: "12345".into(),
        title: "   ".into(),
        sortpubdate: Some("2023/01/15 00:00".into()),
        pubdate: Some("2023 Jan 15".into()),
        edat: None,
        lr: None,
        fulljournalname: Some("Nature Medicine".into()),
        source: Some("Nat Med".into()),
    });

    assert!(row.is_none());
}

#[test]
fn from_pubmed_esummary_entry_uses_edat_for_first_index_date() {
    let row = from_pubmed_esummary_entry(&ESummaryEntry {
        uid: "12345".into(),
        title: "PubMed date".into(),
        sortpubdate: Some("2023/01/15 00:00".into()),
        pubdate: Some("2023 Jan 15".into()),
        edat: Some("2023/01/16 00:00:00".into()),
        lr: Some("2023/01/17 00:00:00".into()),
        fulljournalname: Some("Nature Medicine".into()),
        source: Some("Nat Med".into()),
    })
    .expect("pubmed row should map");

    assert_eq!(
        row.first_index_date,
        Some(NaiveDate::from_ymd_opt(2023, 1, 16).expect("valid date"))
    );
}

#[test]
fn from_pubmed_esummary_entry_falls_back_to_lr_for_first_index_date() {
    let row = from_pubmed_esummary_entry(&ESummaryEntry {
        uid: "12345".into(),
        title: "PubMed date".into(),
        sortpubdate: Some("2023/01/15 00:00".into()),
        pubdate: Some("2023 Jan 15".into()),
        edat: None,
        lr: Some("2023/01/17 00:00:00".into()),
        fulljournalname: Some("Nature Medicine".into()),
        source: Some("Nat Med".into()),
    })
    .expect("pubmed row should map");

    assert_eq!(
        row.first_index_date,
        Some(NaiveDate::from_ymd_opt(2023, 1, 17).expect("valid date"))
    );
}

#[test]
fn from_pubmed_esummary_entry_prefers_edat_over_lr() {
    let row = from_pubmed_esummary_entry(&ESummaryEntry {
        uid: "12345".into(),
        title: "PubMed date".into(),
        sortpubdate: Some("2023/01/15 00:00".into()),
        pubdate: Some("2023 Jan 15".into()),
        edat: Some("2023/01/16 00:00:00".into()),
        lr: Some("2023/01/17 00:00:00".into()),
        fulljournalname: Some("Nature Medicine".into()),
        source: Some("Nat Med".into()),
    })
    .expect("pubmed row should map");

    assert_eq!(
        row.first_index_date,
        Some(NaiveDate::from_ymd_opt(2023, 1, 16).expect("valid date"))
    );
}
