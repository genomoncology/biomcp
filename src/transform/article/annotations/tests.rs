//! Annotation aggregation regression tests.

use super::*;
use crate::entities::article::AnnotationCount;
use crate::sources::pubtator::PubTatorDocument;

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
fn extract_annotations_preserves_first_seen_order_for_equal_counts() {
    let doc: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "pmid": 22663011,
        "passages": [
            {
                "infons": {"type": "title"},
                "text": "Example title",
                "annotations": [
                    {"text": "TP53", "infons": {"type": "Gene"}},
                    {"text": "BRAF", "infons": {"type": "Gene"}},
                    {"text": "TP53", "infons": {"type": "Gene"}},
                    {"text": "BRAF", "infons": {"type": "Gene"}}
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
                text: "TP53".into(),
                count: 2
            },
            AnnotationCount {
                text: "BRAF".into(),
                count: 2
            }
        ]
    );
}

#[test]
fn adopted_and_legacy_details_share_annotation_aggregation() {
    let admitted = br#"{"PubTator3":[{"id":"7","passages":[{"infons":{"type":"title"},"offset":0,"text":"Title","sentences":[],"annotations":[{"id":"a1","infons":{"type":"Gene"},"text":"BRAF","locations":[{"offset":0,"length":4}]},{"id":"a2","infons":{"type":"Gene"},"text":"BRAF","locations":[{"offset":5,"length":4}]}],"relations":[]}]}]}"#;
    let adopted = crate::sources::pubtator::parse_publication_detail(7, admitted)
        .expect("admitted response")
        .expect("admitted detail");
    let document: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "id": "7",
        "passages": [{
            "infons": {"type": "title"},
            "text": "Title",
            "annotations": [
                {"text": "BRAF", "infons": {"type": "Gene"}},
                {"text": "BRAF", "infons": {"type": "Gene"}}
            ]
        }]
    }))
    .expect("legacy document");
    let legacy = crate::sources::pubtator::PubTatorDetail::Legacy {
        requested_pmid: biodata::Pmid::new("7").expect("PMID"),
        document,
    };

    assert_eq!(
        extract_detail_annotations(&adopted),
        extract_detail_annotations(&legacy)
    );
}
