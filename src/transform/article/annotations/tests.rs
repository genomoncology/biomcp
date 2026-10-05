//! Annotation aggregation regression tests.

use super::*;
use crate::entities::article::AnnotationCount;
use crate::sources::pubtator::PubTatorDocument;

fn row(text: &str, count: u32) -> AnnotationCount {
    AnnotationCount {
        text: text.into(),
        count,
        ..Default::default()
    }
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

    let ann = extract_annotations(&doc, false).expect("annotations should exist");
    assert_eq!(ann.genes, vec![row("BRAF", 2), row("TP53", 1)]);
    assert_eq!(ann.mutations, vec![row("V600E", 2)]);
    assert_eq!(ann.diseases, vec![row("melanoma", 1)]);
    assert_eq!(ann.chemicals, vec![row("vemurafenib", 1)]);
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

    let ann = extract_annotations(&doc, false).expect("annotations should exist");
    assert_eq!(ann.genes, vec![row("TP53", 2), row("BRAF", 2)]);
}

#[test]
fn extract_annotations_reads_identifier_and_namespace() {
    let doc: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "pmid": 30738221,
        "passages": [
            {
                "infons": {"type": "title"},
                "text": "KRAS G12C in NSCLC",
                "annotations": [
                    {
                        "text": "KRAS",
                        "infons": {"type": "Gene", "identifier": "3845", "database": "ncbi_gene"}
                    },
                    {
                        "text": "NSCLC",
                        "infons": {"type": "Disease", "identifier": "MESH:D002289"}
                    },
                    {
                        "text": "G12C",
                        "infons": {
                            "type": "Variant",
                            "identifier": "tmVar:p|SUB|G|12|C",
                            "hgvs": "p.G12C",
                            "rsid": "rs121913530"
                        }
                    },
                    {
                        "text": "MELANOMA",
                        "infons": {"type": "Disease", "identifier": "OMIM:155601"}
                    }
                ]
            }
        ]
    }))
    .expect("valid JSON");

    let ann = extract_annotations(&doc, false).expect("annotations should exist");
    assert_eq!(
        ann.genes,
        vec![AnnotationCount {
            text: "KRAS".into(),
            count: 1,
            namespace: Some("NCBIGene".into()),
            identifier: Some("3845".into()),
            ..Default::default()
        }]
    );
    assert_eq!(
        ann.diseases,
        vec![
            AnnotationCount {
                text: "NSCLC".into(),
                count: 1,
                namespace: Some("MESH".into()),
                identifier: Some("MESH:D002289".into()),
                ..Default::default()
            },
            AnnotationCount {
                text: "MELANOMA".into(),
                count: 1,
                namespace: Some("OMIM".into()),
                identifier: Some("OMIM:155601".into()),
                ..Default::default()
            }
        ]
    );
    // The variant's typed-back identifier is the rsID, not the tmVar composite.
    assert_eq!(
        ann.mutations,
        vec![AnnotationCount {
            text: "G12C".into(),
            count: 1,
            namespace: Some("rsID".into()),
            identifier: Some("rs121913530".into()),
            ..Default::default()
        }]
    );
}

#[test]
fn extract_annotations_falls_back_to_hgvs_without_rsid() {
    let doc: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "pmid": 1,
        "passages": [
            {
                "infons": {"type": "abstract"},
                "text": "text",
                "annotations": [
                    {
                        "text": "V600E",
                        "infons": {
                            "type": "Mutation",
                            "identifier": "tmVar:p|SUB|V|600|E",
                            "hgvs": "NM_004333.6:c.1799T>A"
                        }
                    },
                    {
                        "text": "G13C",
                        "infons": {
                            "type": "Mutation",
                            "identifier": "tmVar:p|SUB|G|13|C",
                            "rsids": ["rs121913535"]
                        }
                    },
                    {
                        "text": "G12A",
                        "infons": {"type": "Mutation", "identifier": "tmVar:p|SUB|G|12|A"}
                    }
                ]
            }
        ]
    }))
    .expect("valid JSON");

    let ann = extract_annotations(&doc, false).expect("annotations should exist");
    assert_eq!(
        ann.mutations,
        vec![
            AnnotationCount {
                text: "V600E".into(),
                count: 1,
                namespace: Some("HGVS".into()),
                identifier: Some("NM_004333.6:c.1799T>A".into()),
                ..Default::default()
            },
            AnnotationCount {
                text: "G13C".into(),
                count: 1,
                namespace: Some("rsID".into()),
                identifier: Some("rs121913535".into()),
                ..Default::default()
            },
            // No rsID and no HGVS means no identifier to carry.
            row("G12A", 1),
        ]
    );
}

#[test]
fn extract_annotations_ignores_placeholder_identifiers() {
    let doc: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "pmid": 2,
        "passages": [
            {
                "infons": {"type": "abstract"},
                "text": "text",
                "annotations": [
                    {"text": "EVT801", "infons": {"type": "Chemical", "identifier": "-"}},
                    {"text": "mystery", "infons": {"type": "Disease", "identifier": "unprefixed"}}
                ]
            }
        ]
    }))
    .expect("valid JSON");

    let ann = extract_annotations(&doc, false).expect("annotations should exist");
    assert_eq!(ann.chemicals, vec![row("EVT801", 1)]);
    assert_eq!(ann.diseases, vec![row("mystery", 1)]);
}

#[test]
fn same_text_with_different_identifiers_stays_separate() {
    let doc: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "pmid": 3,
        "passages": [
            {
                "infons": {"type": "abstract"},
                "text": "text",
                "annotations": [
                    {"text": "cancer", "infons": {"type": "Disease", "identifier": "MESH:D009369"}},
                    {"text": "Cancer", "infons": {"type": "Disease", "identifier": "MESH:D001943"}},
                    {"text": "Cancer", "infons": {"type": "Disease", "identifier": "MESH:D009369"}}
                ]
            }
        ]
    }))
    .expect("valid JSON");

    let ann = extract_annotations(&doc, false).expect("annotations should exist");
    assert_eq!(
        ann.diseases,
        vec![
            AnnotationCount {
                text: "cancer".into(),
                count: 2,
                namespace: Some("MESH".into()),
                identifier: Some("MESH:D009369".into()),
                ..Default::default()
            },
            AnnotationCount {
                text: "Cancer".into(),
                count: 1,
                namespace: Some("MESH".into()),
                identifier: Some("MESH:D001943".into()),
                ..Default::default()
            }
        ]
    );
}

#[test]
fn positions_land_only_when_requested() {
    let doc: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "pmid": 4,
        "passages": [
            {
                "infons": {"type": "title"},
                "text": "KRAS in cancer",
                "annotations": [
                    {
                        "text": "KRAS",
                        "locations": [{"offset": 0, "length": 4}],
                        "infons": {"type": "Gene", "identifier": "3845"}
                    },
                    {
                        "text": "KRAS",
                        "locations": [{"offset": 24, "length": 4}],
                        "infons": {"type": "Gene", "identifier": "3845"}
                    }
                ]
            }
        ]
    }))
    .expect("valid JSON");

    let compact = extract_annotations(&doc, false).expect("annotations should exist");
    assert_eq!(compact.genes[0].positions, Vec::new());

    let full = extract_annotations(&doc, true).expect("annotations should exist");
    assert_eq!(
        full.genes[0].positions,
        vec![
            AnnotationPosition {
                offset: 0,
                length: 4
            },
            AnnotationPosition {
                offset: 24,
                length: 4
            }
        ]
    );
    assert_eq!(full.genes[0].count, 2);
}
