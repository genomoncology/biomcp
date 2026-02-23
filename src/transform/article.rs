use std::collections::HashMap;
use std::sync::OnceLock;

use regex::Regex;

use crate::entities::article::{AnnotationCount, Article, ArticleAnnotations, ArticleSearchResult};
use crate::sources::europepmc::EuropePmcResult;
use crate::sources::pubtator::PubTatorDocument;

fn truncate_utf8(s: &str, max_bytes: usize, suffix: &str) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }

    let mut boundary = max_bytes;
    while boundary > 0 && !s.is_char_boundary(boundary) {
        boundary -= 1;
    }
    let mut out = s[..boundary].trim_end().to_string();
    out.push_str(suffix);
    out
}

fn decode_html_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}

fn strip_inline_html_tags(value: &str) -> String {
    static HTML_TAG_RE: OnceLock<Regex> = OnceLock::new();
    let re = HTML_TAG_RE.get_or_init(|| Regex::new(r"(?is)<[^>]+>").expect("valid regex"));
    re.replace_all(value, "").to_string()
}

fn clean_title(value: &str) -> String {
    strip_inline_html_tags(&decode_html_entities(value))
        .trim()
        .to_string()
}

fn clean_abstract(value: &str) -> String {
    strip_inline_html_tags(&decode_html_entities(value))
        .trim()
        .to_string()
}

pub fn truncate_title(title: &str) -> String {
    const MAX_TITLE_BYTES: usize = 60;
    truncate_utf8(&clean_title(title), MAX_TITLE_BYTES, "…")
}

pub fn truncate_abstract(text: &str) -> String {
    const MAX_ABSTRACT_BYTES: usize = 1500;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.len() <= MAX_ABSTRACT_BYTES {
        return trimmed.to_string();
    }

    let short = truncate_utf8(trimmed, MAX_ABSTRACT_BYTES, "...");
    let total = trimmed.chars().count();
    format!("{short}\n\n(truncated, {total} chars total)")
}

pub fn truncate_authors(authors: &[String]) -> Vec<String> {
    if authors.len() <= 4 {
        return authors.to_vec();
    }
    match (authors.first(), authors.last()) {
        (Some(first), Some(last)) if first != last => vec![first.clone(), last.clone()],
        _ => authors.iter().take(2).cloned().collect(),
    }
}

pub fn from_pubtator_document(doc: &PubTatorDocument) -> Article {
    let mut title: Option<String> = None;
    let mut abstract_text: Option<String> = None;
    for p in &doc.passages {
        let kind = p
            .infons
            .as_ref()
            .and_then(|i| i.kind.as_deref())
            .unwrap_or("");
        let text = p.text.as_deref().unwrap_or("").trim();
        if text.is_empty() {
            continue;
        }
        match kind {
            "title" if title.is_none() => title = Some(text.to_string()),
            "abstract" if abstract_text.is_none() => abstract_text = Some(text.to_string()),
            _ => {}
        }
    }

    Article {
        pmid: doc.pmid.map(|v| v.to_string()),
        pmcid: doc.pmcid.clone(),
        doi: None,
        title: title.unwrap_or_default().trim().to_string(),
        authors: truncate_authors(&doc.authors),
        journal: doc
            .journal
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        date: doc
            .date
            .as_deref()
            .and_then(|d| d.get(0..10))
            .map(|s| s.to_string()),
        citation_count: None,
        publication_type: None,
        open_access: None,
        abstract_text: abstract_text
            .map(|t| truncate_abstract(&t))
            .filter(|t| !t.is_empty()),
        full_text_path: None,
        full_text_note: None,
        annotations: None,
        pubtator_fallback: false,
    }
}

fn parse_citation_count(value: Option<&serde_json::Value>) -> Option<u64> {
    let value = value?;
    match value {
        serde_json::Value::Number(n) => n.as_u64(),
        serde_json::Value::String(s) => s.trim().parse::<u64>().ok(),
        _ => None,
    }
}

fn normalize_publication_type(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    let mapped = if lower.contains("meta-analysis") {
        "Meta-Analysis".to_string()
    } else if lower.contains("review") {
        "Review".to_string()
    } else if lower.contains("case report") {
        "Case Report".to_string()
    } else if lower.contains("research-article") || lower.contains("journal article") {
        "Research Article".to_string()
    } else {
        trimmed.to_string()
    };
    Some(mapped)
}

fn collect_publication_types_from_value(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::String(s) => {
            for token in s.split(';') {
                let token = token.trim();
                if !token.is_empty() {
                    out.push(token.to_string());
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                if let Some(text) = item.as_str().map(str::trim).filter(|v| !v.is_empty()) {
                    out.push(text.to_string());
                    continue;
                }
                if let Some(text) = item
                    .as_object()
                    .and_then(|o| o.get("name"))
                    .and_then(|v| v.as_str())
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                {
                    out.push(text.to_string());
                    continue;
                }
                collect_publication_types_from_value(item, out);
            }
        }
        serde_json::Value::Object(obj) => {
            for value in obj.values() {
                collect_publication_types_from_value(value, out);
            }
        }
        _ => {}
    };
}

fn publication_types(hit: &EuropePmcResult) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(value) = hit.pub_type.as_ref() {
        collect_publication_types_from_value(value, &mut out);
    }
    if let Some(value) = hit.pub_type_list.as_ref() {
        collect_publication_types_from_value(value, &mut out);
    }

    let mut deduped = Vec::new();
    for value in out {
        if deduped
            .iter()
            .any(|v: &String| v.eq_ignore_ascii_case(&value))
        {
            continue;
        }
        deduped.push(value);
    }
    deduped
}

fn parse_publication_type(hit: &EuropePmcResult) -> Option<String> {
    publication_types(hit)
        .into_iter()
        .find_map(|v| normalize_publication_type(&v))
}

fn is_retracted_publication(hit: &EuropePmcResult) -> bool {
    publication_types(hit)
        .into_iter()
        .any(|value| value.to_ascii_lowercase().contains("retracted publication"))
}

fn parse_open_access(value: Option<&serde_json::Value>) -> Option<bool> {
    let value = value?;
    match value {
        serde_json::Value::Bool(v) => Some(*v),
        serde_json::Value::String(v) => match v.trim().to_ascii_uppercase().as_str() {
            "Y" | "YES" | "TRUE" | "1" => Some(true),
            "N" | "NO" | "FALSE" | "0" => Some(false),
            _ => None,
        },
        serde_json::Value::Number(v) => v.as_u64().map(|n| n > 0),
        _ => None,
    }
}

fn split_author_string(value: &str) -> Vec<String> {
    let v = value.trim();
    if v.is_empty() {
        return vec![];
    }
    v.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .take(10)
        .collect()
}

pub fn from_europepmc_result(hit: &EuropePmcResult) -> Article {
    Article {
        pmid: hit.pmid.clone(),
        pmcid: hit.pmcid.clone(),
        doi: hit.doi.clone(),
        title: clean_title(hit.title.as_deref().unwrap_or_default()),
        authors: hit
            .author_string
            .as_deref()
            .map(split_author_string)
            .unwrap_or_default(),
        journal: hit
            .journal_title
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        date: hit
            .first_publication_date
            .as_ref()
            .or(hit.pub_year.as_ref())
            .map(|s| s.get(0..10).unwrap_or(s).to_string()),
        citation_count: parse_citation_count(hit.cited_by_count.as_ref()),
        publication_type: parse_publication_type(hit),
        open_access: parse_open_access(hit.is_open_access.as_ref()),
        abstract_text: hit
            .abstract_text
            .as_deref()
            .map(clean_abstract)
            .map(|text| truncate_abstract(&text))
            .filter(|text| !text.is_empty()),
        full_text_path: None,
        full_text_note: None,
        annotations: None,
        pubtator_fallback: false,
    }
}

pub fn merge_europepmc_metadata(article: &mut Article, hit: &EuropePmcResult) {
    if article.doi.is_none() {
        article.doi = hit.doi.clone();
    }
    if article.pmcid.is_none() {
        article.pmcid = hit.pmcid.clone();
    }
    if article.journal.is_none() {
        article.journal = hit
            .journal_title
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
    }
    if article.date.is_none() {
        article.date = hit
            .first_publication_date
            .as_ref()
            .or(hit.pub_year.as_ref())
            .map(|s| s.get(0..10).unwrap_or(s).to_string());
    }

    article.citation_count = parse_citation_count(hit.cited_by_count.as_ref());
    article.publication_type = parse_publication_type(hit);
    article.open_access = parse_open_access(hit.is_open_access.as_ref());
    if article.abstract_text.is_none() {
        article.abstract_text = hit
            .abstract_text
            .as_deref()
            .map(clean_abstract)
            .map(|text| truncate_abstract(&text))
            .filter(|text| !text.is_empty());
    }
}

pub fn from_europepmc_search_result(hit: &EuropePmcResult) -> Option<ArticleSearchResult> {
    let pmid = hit
        .pmid
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())?
        .to_string();
    Some(ArticleSearchResult {
        pmid,
        title: truncate_title(hit.title.as_deref().unwrap_or_default()),
        journal: hit
            .journal_title
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        date: hit
            .first_publication_date
            .as_ref()
            .or(hit.pub_year.as_ref())
            .map(|s| s.get(0..10).unwrap_or(s).to_string()),
        citation_count: parse_citation_count(hit.cited_by_count.as_ref()),
        is_retracted: is_retracted_publication(hit),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnnotationKind {
    Gene,
    Disease,
    Chemical,
    Mutation,
}

fn annotation_kind(kind: &str) -> Option<AnnotationKind> {
    let k = kind.trim().to_ascii_lowercase();
    if k.is_empty() {
        return None;
    }
    if k.contains("gene") {
        return Some(AnnotationKind::Gene);
    }
    if k.contains("disease") {
        return Some(AnnotationKind::Disease);
    }
    if k.contains("chemical") || k.contains("drug") {
        return Some(AnnotationKind::Chemical);
    }
    if k.contains("mutation") || k.contains("variant") {
        return Some(AnnotationKind::Mutation);
    }
    None
}

fn push_annotation_count(map: &mut HashMap<String, (String, u32)>, text: &str) {
    let t = text.trim();
    if t.is_empty() || t.len() > 128 {
        return;
    }
    let key = t.to_ascii_lowercase();
    let entry = map.entry(key).or_insert_with(|| (t.to_string(), 0));
    entry.1 += 1;
}

fn finalize_counts(map: HashMap<String, (String, u32)>) -> Vec<AnnotationCount> {
    let mut out = map
        .into_values()
        .map(|(text, count)| AnnotationCount { text, count })
        .collect::<Vec<_>>();
    out.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.text.cmp(&b.text)));
    out.truncate(8);
    out
}

pub fn extract_annotations(doc: &PubTatorDocument) -> Option<ArticleAnnotations> {
    let mut genes: HashMap<String, (String, u32)> = HashMap::new();
    let mut diseases: HashMap<String, (String, u32)> = HashMap::new();
    let mut chemicals: HashMap<String, (String, u32)> = HashMap::new();
    let mut mutations: HashMap<String, (String, u32)> = HashMap::new();

    for passage in &doc.passages {
        for ann in &passage.annotations {
            let Some(text) = ann.text.as_deref() else {
                continue;
            };
            let Some(kind) = ann
                .infons
                .as_ref()
                .and_then(|i| i.kind.as_deref())
                .and_then(annotation_kind)
            else {
                continue;
            };

            match kind {
                AnnotationKind::Gene => push_annotation_count(&mut genes, text),
                AnnotationKind::Disease => push_annotation_count(&mut diseases, text),
                AnnotationKind::Chemical => push_annotation_count(&mut chemicals, text),
                AnnotationKind::Mutation => push_annotation_count(&mut mutations, text),
            }
        }
    }

    let annotations = ArticleAnnotations {
        genes: finalize_counts(genes),
        diseases: finalize_counts(diseases),
        chemicals: finalize_counts(chemicals),
        mutations: finalize_counts(mutations),
    };

    if annotations.genes.is_empty()
        && annotations.diseases.is_empty()
        && annotations.chemicals.is_empty()
        && annotations.mutations.is_empty()
    {
        None
    } else {
        Some(annotations)
    }
}

pub fn extract_text_from_xml(xml: &str) -> String {
    // Best-effort tag stripping (good enough for caching / basic readability).
    let mut out = String::with_capacity(xml.len().min(32_000));
    let mut in_tag = false;

    for ch in xml.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if in_tag => {}
            _ => out.push(ch),
        }
    }

    out = out.replace("\r\n", "\n");
    out = out.replace('\r', "\n");
    while out.contains("\n\n\n") {
        out = out.replace("\n\n\n", "\n\n");
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_title_truncates_on_utf8_boundary() {
        let title = "€".repeat(100);
        let out = truncate_title(&title);
        assert!(out.ends_with('…'));
        assert!(out.len() <= 63);
    }

    #[test]
    fn truncate_title_strips_inline_html_and_entities() {
        let title = "KRAS&lt;sup&gt;G12C&lt;/sup&gt; and <i>melanoma</i>";
        let out = truncate_title(title);
        assert!(out.contains("KRAS"));
        assert!(!out.contains("&lt;"));
        assert!(!out.contains("<i>"));
    }

    #[test]
    fn truncate_abstract_keeps_full_text_until_limit() {
        let text = "Sentence one. Sentence two. Sentence three.";
        let out = truncate_abstract(text);
        assert_eq!(out, text);
    }

    #[test]
    fn truncate_authors_first_last() {
        let authors = vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
            "E".to_string(),
        ];
        assert_eq!(truncate_authors(&authors), vec!["A", "E"]);
    }

    #[test]
    fn extract_annotations_counts_mentions() {
        let doc: PubTatorDocument = serde_json::from_value(serde_json::json!({
            "pmid": 123,
            "pmcid": "PMC1",
            "date": "2026-02-05",
            "journal": "Test",
            "authors": ["A"],
            "passages": [
                {
                    "infons": {"type": "title"},
                    "text": "BRAF V600E in melanoma",
                    "annotations": [
                        {"text": "BRAF", "infons": {"type": "Gene"}},
                        {"text": "V600E", "infons": {"type": "Mutation"}},
                        {"text": "melanoma", "infons": {"type": "Disease"}}
                    ]
                },
                {
                    "infons": {"type": "abstract"},
                    "text": "Vemurafenib targets BRAF V600E",
                    "annotations": [
                        {"text": "BRAF", "infons": {"type": "Gene"}},
                        {"text": "TP53", "infons": {"type": "Gene"}},
                        {"text": "V600E", "infons": {"type": "Mutation"}},
                        {"text": "vemurafenib", "infons": {"type": "Chemical"}}
                    ]
                }
            ]
        }))
        .expect("valid JSON");

        let ann = extract_annotations(&doc).expect("annotations should exist");
        assert_eq!(
            ann.genes,
            vec![
                AnnotationCount {
                    text: "BRAF".into(),
                    count: 2
                },
                AnnotationCount {
                    text: "TP53".into(),
                    count: 1
                }
            ]
        );
        assert_eq!(
            ann.mutations,
            vec![AnnotationCount {
                text: "V600E".into(),
                count: 2
            }]
        );
        assert_eq!(
            ann.diseases,
            vec![AnnotationCount {
                text: "melanoma".into(),
                count: 1
            }]
        );
        assert_eq!(
            ann.chemicals,
            vec![AnnotationCount {
                text: "vemurafenib".into(),
                count: 1
            }]
        );
    }

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
        assert_eq!(article.authors, vec!["Author A", "Author E"]);
        assert!(!article.pubtator_fallback);
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
        assert!(row.is_retracted);
    }
}
