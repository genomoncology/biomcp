//! Mutation identity tests: which identifier a variant row carries.

use super::super::extract_annotations;
use super::row;
use crate::entities::article::AnnotationCount;
use crate::sources::pubtator::PubTatorDocument;

#[test]
fn rows_with_gene_and_change_prefer_the_gene_qualified_form() {
    let doc: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "pmid": 30738221,
        "passages": [
            {
                "infons": {"type": "title"},
                "text": "KRAS G12A, G12C, G12D, G12V and G13C in NSCLC",
                "annotations": [
                    {"text": "KRAS", "infons": {"type": "Gene", "identifier": "3845"}},
                    {
                        "text": "G12A",
                        "infons": {
                            "type": "Variant",
                            "identifier": "tmVar:p|SUB|G|12|A",
                            "hgvs": "p.G12A",
                            "rsid": "rs121913529",
                            "gene_id": 3845
                        }
                    },
                    {
                        "text": "G12C",
                        "infons": {
                            "type": "Variant",
                            "identifier": "tmVar:p|SUB|G|12|C",
                            "hgvs": "p.G12C",
                            "rsid": "rs121913530",
                            "gene_id": 3845
                        }
                    },
                    {
                        "text": "G12D",
                        "infons": {
                            "type": "Variant",
                            "identifier": "tmVar:p|SUB|G|12|D",
                            "hgvs": "p.G12D",
                            "rsid": "rs121913529",
                            "gene_ids": [3845]
                        }
                    },
                    {
                        "text": "G12V",
                        "infons": {
                            "type": "Variant",
                            "identifier": "tmVar:p|SUB|G|12|V",
                            "hgvs": "p.G12V",
                            "rsid": "rs121913529",
                            "gene_id": 3845
                        }
                    },
                    {
                        "text": "G13C",
                        "infons": {
                            "type": "Variant",
                            "identifier": "tmVar:p|SUB|G|13|C",
                            "hgvs": "p.G13C",
                            "rsids": ["rs121913535"]
                        }
                    }
                ]
            }
        ]
    }))
    .expect("valid JSON");

    let ann = extract_annotations(&doc, false).expect("annotations should exist");
    // Every row with a gene id and a protein change carries the
    // gene-qualified form, so the multi-allele rs121913529 rows and the
    // single-allele rs121913530 row all name their own allele. G13C carries
    // no gene id, so its rsID, which this document shows naming one change,
    // keeps the link.
    assert_eq!(
        ann.mutations,
        vec![
            AnnotationCount {
                text: "G12A".into(),
                count: 1,
                namespace: Some("HGVS".into()),
                identifier: Some("KRAS p.G12A".into()),
                ..Default::default()
            },
            AnnotationCount {
                text: "G12C".into(),
                count: 1,
                namespace: Some("HGVS".into()),
                identifier: Some("KRAS p.G12C".into()),
                ..Default::default()
            },
            AnnotationCount {
                text: "G12D".into(),
                count: 1,
                namespace: Some("HGVS".into()),
                identifier: Some("KRAS p.G12D".into()),
                ..Default::default()
            },
            AnnotationCount {
                text: "G12V".into(),
                count: 1,
                namespace: Some("HGVS".into()),
                identifier: Some("KRAS p.G12V".into()),
                ..Default::default()
            },
            AnnotationCount {
                text: "G13C".into(),
                count: 1,
                namespace: Some("rsID".into()),
                identifier: Some("rs121913535".into()),
                ..Default::default()
            }
        ]
    );
}

#[test]
fn single_mention_allele_of_a_multi_allele_rsid_carries_the_gene_form() {
    let doc: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "pmid": 8,
        "passages": [
            {
                "infons": {"type": "title"},
                "text": "KRAS G12A in NSCLC",
                "annotations": [
                    {"text": "KRAS", "infons": {"type": "Gene", "identifier": "3845"}},
                    {
                        "text": "G12A",
                        "infons": {
                            "type": "Variant",
                            "identifier": "tmVar:p|SUB|G|12|A",
                            "hgvs": "p.G12A",
                            "rsid": "rs121913529",
                            "gene_id": 3845
                        }
                    }
                ]
            }
        ]
    }))
    .expect("valid JSON");

    // This document shows rs121913529 naming one change, yet that rsID opens
    // G12D, so the row names its own allele with the gene-qualified form
    // instead of the rsID.
    let ann = extract_annotations(&doc, false).expect("annotations should exist");
    assert_eq!(
        ann.mutations,
        vec![AnnotationCount {
            text: "G12A".into(),
            count: 1,
            namespace: Some("HGVS".into()),
            identifier: Some("KRAS p.G12A".into()),
            ..Default::default()
        }]
    );
}

#[test]
fn no_rsid_row_with_gene_and_change_gains_the_gene_qualified_form() {
    let doc: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "pmid": 10,
        "passages": [
            {
                "infons": {"type": "title"},
                "text": "KRAS G12A in NSCLC",
                "annotations": [
                    {"text": "KRAS", "infons": {"type": "Gene", "identifier": "3845"}},
                    {
                        "text": "G12A",
                        "infons": {
                            "type": "Variant",
                            "identifier": "tmVar:p|SUB|G|12|A",
                            "hgvs": "p.G12A",
                            "gene_id": 3845
                        }
                    }
                ]
            }
        ]
    }))
    .expect("valid JSON");

    // The row carries no rsID, so the old rule fell through to the bare
    // protein change `p.G12A`, which is shorthand, not exact input, and
    // rendered as the mention-text command. The row-own-data rule upgrades
    // it to the gene-qualified exact form.
    let ann = extract_annotations(&doc, false).expect("annotations should exist");
    assert_eq!(
        ann.mutations,
        vec![AnnotationCount {
            text: "G12A".into(),
            count: 1,
            namespace: Some("HGVS".into()),
            identifier: Some("KRAS p.G12A".into()),
            ..Default::default()
        }]
    );
}

#[test]
fn one_allele_written_two_ways_stays_one_rsid_change() {
    let doc: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "pmid": 9,
        "passages": [
            {
                "infons": {"type": "abstract"},
                "text": "G12D and Gly12Asp",
                "annotations": [
                    {
                        "text": "G12D",
                        "infons": {
                            "type": "Mutation",
                            "identifier": "tmVar:p|SUB|G|12|D",
                            "hgvs": "p.G12D",
                            "rsid": "rs121913529"
                        }
                    },
                    {
                        "text": "Gly12Asp",
                        "infons": {
                            "type": "Mutation",
                            "identifier": "tmVar:p|SUB|Gly|12|Asp",
                            "hgvs": "p.Gly12Asp",
                            "rsid": "rs121913529"
                        }
                    }
                ]
            }
        ]
    }))
    .expect("valid JSON");

    // One allele written two ways is still one change, so the rsID keeps its
    // link for both rows. The document annotates no gene, so these rows take
    // the rsID fallback path.
    let ann = extract_annotations(&doc, false).expect("annotations should exist");
    assert_eq!(
        ann.mutations,
        vec![
            AnnotationCount {
                text: "G12D".into(),
                count: 1,
                namespace: Some("rsID".into()),
                identifier: Some("rs121913529".into()),
                ..Default::default()
            },
            AnnotationCount {
                text: "Gly12Asp".into(),
                count: 1,
                namespace: Some("rsID".into()),
                identifier: Some("rs121913529".into()),
                ..Default::default()
            }
        ]
    );
}

#[test]
fn multi_allele_rsid_without_gene_symbol_carries_no_identifier() {
    let doc: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "pmid": 5,
        "passages": [
            {
                "infons": {"type": "abstract"},
                "text": "G12A and G12D",
                "annotations": [
                    {
                        "text": "G12A",
                        "infons": {
                            "type": "Mutation",
                            "identifier": "tmVar:p|SUB|G|12|A",
                            "hgvs": "p.G12A",
                            "rsid": "rs121913529",
                            "gene_id": 3845
                        }
                    },
                    {
                        "text": "G12D",
                        "infons": {
                            "type": "Mutation",
                            "identifier": "tmVar:p|SUB|G|12|D",
                            "hgvs": "p.G12D",
                            "rsid": "rs121913529",
                            "gene_id": 3845
                        }
                    }
                ]
            }
        ]
    }))
    .expect("valid JSON");

    // The document annotates no gene for NCBI Gene 3845, so no typed-back
    // allele form exists and the rows keep the mention text alone.
    let ann = extract_annotations(&doc, false).expect("annotations should exist");
    assert_eq!(ann.mutations, vec![row("G12A", 1), row("G12D", 1)]);
}

#[test]
fn multi_allele_rsid_with_coding_hgvs_keeps_the_coding_form() {
    let doc: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "pmid": 6,
        "passages": [
            {
                "infons": {"type": "abstract"},
                "text": "coding aliases",
                "annotations": [
                    {
                        "text": "G12A",
                        "infons": {
                            "type": "Mutation",
                            "identifier": "tmVar:c|SUB|G|35|C",
                            "hgvs": "NM_033360.4:c.35G>C",
                            "rsid": "rs121913529",
                            "gene_id": 3845
                        }
                    },
                    {
                        "text": "G12D",
                        "infons": {
                            "type": "Mutation",
                            "identifier": "tmVar:c|SUB|G|35|A",
                            "hgvs": "NM_033360.4:c.35G>A",
                            "rsid": "rs121913529",
                            "gene_id": 3845
                        }
                    }
                ]
            }
        ]
    }))
    .expect("valid JSON");

    // A coding HGVS names the allele by itself, so the shared rsID drops
    // without a gene-qualified form.
    let ann = extract_annotations(&doc, false).expect("annotations should exist");
    assert_eq!(
        ann.mutations,
        vec![
            AnnotationCount {
                text: "G12A".into(),
                count: 1,
                namespace: Some("HGVS".into()),
                identifier: Some("NM_033360.4:c.35G>C".into()),
                ..Default::default()
            },
            AnnotationCount {
                text: "G12D".into(),
                count: 1,
                namespace: Some("HGVS".into()),
                identifier: Some("NM_033360.4:c.35G>A".into()),
                ..Default::default()
            }
        ]
    );
}

#[test]
fn gene_symbol_prefers_the_most_mentioned_gene_text() {
    let doc: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "pmid": 7,
        "passages": [
            {
                "infons": {"type": "abstract"},
                "text": "MYC, myc again, G12A and G12D",
                "annotations": [
                    {"text": "myc", "infons": {"type": "Gene", "identifier": "4609"}},
                    {"text": "MYC", "infons": {"type": "Gene", "identifier": "4609"}},
                    {"text": "MYC", "infons": {"type": "Gene", "identifier": "4609"}},
                    {
                        "text": "G12A",
                        "infons": {
                            "type": "Mutation",
                            "identifier": "tmVar:p|SUB|G|12|A",
                            "hgvs": "p.G12A",
                            "rsid": "rs121913529",
                            "gene_id": 4609
                        }
                    },
                    {
                        "text": "G12D",
                        "infons": {
                            "type": "Mutation",
                            "identifier": "tmVar:p|SUB|G|12|D",
                            "hgvs": "p.G12D",
                            "rsid": "rs121913529",
                            "gene_id": 4609
                        }
                    }
                ]
            }
        ]
    }))
    .expect("valid JSON");

    let ann = extract_annotations(&doc, false).expect("annotations should exist");
    assert_eq!(
        ann.mutations
            .iter()
            .map(|m| m.identifier.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("MYC p.G12A"), Some("MYC p.G12D")]
    );
}
