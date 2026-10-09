//! Mutation identity tests: which identifier a variant row carries.

use super::super::{
    extract_annotations, extract_annotations_with_official_symbols,
    gene_ids_needing_official_symbols,
};
use super::row;
use crate::entities::article::AnnotationCount;
use crate::sources::pubtator::PubTatorDocument;
use std::collections::HashMap;

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

#[test]
fn spelled_gene_mentions_pick_the_most_mentioned_symbol_shaped_text() {
    let doc: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "pmid": 11,
        "passages": [
            {
                "infons": {"type": "abstract"},
                "text": "K-ras, again K-ras, and once KRAS, with G12A",
                "annotations": [
                    {"text": "K-ras", "infons": {"type": "Gene", "identifier": "3845"}},
                    {"text": "K-ras", "infons": {"type": "Gene", "identifier": "3845"}},
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

    // "K-ras" is the most-mentioned text, but the row must not fall back to
    // rs121913529, which opens G12D. The most-mentioned text that passes the
    // symbol check heads the form, so the row names its own allele. On the
    // pre-2034 rule this test fails: the most-mentioned text "K-ras" fails
    // the symbol check and the row carries the rsID.
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
fn gene_without_symbol_shaped_mention_uses_the_official_symbol() {
    let doc: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "pmid": 12,
        "passages": [
            {
                "infons": {"type": "abstract"},
                "text": "K-Ras and K-RAS spellings with G12C, G12V and G12D",
                "annotations": [
                    {"text": "K-Ras", "infons": {"type": "Gene", "identifier": "3845"}},
                    {"text": "K-RAS", "infons": {"type": "Gene", "identifier": "3845"}},
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
                        "text": "G12D",
                        "infons": {
                            "type": "Variant",
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

    // Every mention of NCBI Gene 3845 fails the symbol check, so the gene
    // identifier's official symbol (resolved through MyGene by the caller)
    // heads the form. Without it the G12C row falls to its rsID and the
    // shared-rsID rows lose their identifiers.
    let official = HashMap::from([(3845u64, "KRAS".to_string())]);
    let ann = extract_annotations_with_official_symbols(&doc, false, &official)
        .expect("annotations should exist");
    assert_eq!(
        ann.mutations
            .iter()
            .map(|m| (m.text.as_str(), m.identifier.as_deref()))
            .collect::<Vec<_>>(),
        vec![
            ("G12C", Some("KRAS p.G12C")),
            ("G12V", Some("KRAS p.G12V")),
            ("G12D", Some("KRAS p.G12D"))
        ]
    );

    // The document names both G12V and G12D under rs121913529, so with no
    // official symbol the old fallbacks stand: no gene form exists, the
    // shared rsID loses its link, and only the rsID this document shows
    // naming one change (rs121913530 → G12C) keeps it.
    let ann = extract_annotations(&doc, false).expect("annotations should exist");
    assert_eq!(
        ann.mutations,
        vec![
            AnnotationCount {
                text: "G12C".into(),
                count: 1,
                namespace: Some("rsID".into()),
                identifier: Some("rs121913530".into()),
                ..Default::default()
            },
            row("G12V", 1),
            row("G12D", 1)
        ]
    );
}

#[test]
fn official_symbol_that_is_not_symbol_shaped_builds_no_form() {
    let doc: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "pmid": 13,
        "passages": [
            {
                "infons": {"type": "abstract"},
                "text": "K-RAS G12C",
                "annotations": [
                    {"text": "K-RAS", "infons": {"type": "Gene", "identifier": "3845"}},
                    {
                        "text": "G12C",
                        "infons": {
                            "type": "Variant",
                            "identifier": "tmVar:p|SUB|G|12|C",
                            "hgvs": "p.G12C",
                            "rsid": "rs121913530",
                            "gene_id": 3845
                        }
                    }
                ]
            }
        ]
    }))
    .expect("valid JSON");

    // An official symbol BioMCP cannot parse as a gene token must not build
    // a form `get variant` would refuse; the row keeps the rsID this
    // document shows naming one change.
    let official = HashMap::from([(3845u64, "K-RAS2".to_string())]);
    let ann = extract_annotations_with_official_symbols(&doc, false, &official)
        .expect("annotations should exist");
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
fn official_symbol_lookups_target_only_mutation_linked_spelled_genes() {
    let spelled: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "pmid": 14,
        "passages": [
            {
                "infons": {"type": "abstract"},
                "text": "K-RAS G12C and RUNX3 loss",
                "annotations": [
                    {"text": "K-RAS", "infons": {"type": "Gene", "identifier": "3845"}},
                    {"text": "RUNX3", "infons": {"type": "Gene", "identifier": "864"}},
                    {
                        "text": "G12C",
                        "infons": {
                            "type": "Variant",
                            "identifier": "tmVar:p|SUB|G|12|C",
                            "hgvs": "p.G12C",
                            "rsid": "rs121913530",
                            "gene_id": 3845
                        }
                    }
                ]
            }
        ]
    }))
    .expect("valid JSON");

    // Only NCBI Gene 3845 needs a lookup: a mutation row pairs it with a
    // protein change while its every mention fails the symbol check.
    // RUNX3 spells a usable symbol, and no mutation row carries it.
    assert_eq!(gene_ids_needing_official_symbols(&spelled), vec![3845]);

    // A symbol-shaped mention answers locally, so no lookup is issued even
    // for a mutation-linked gene.
    let symbol_shaped: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "pmid": 15,
        "passages": [
            {
                "infons": {"type": "abstract"},
                "text": "KRAS G12A",
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
    assert_eq!(
        gene_ids_needing_official_symbols(&symbol_shaped),
        Vec::<u64>::new()
    );

    // A row whose change is coding, not protein, builds no gene-plus-change
    // form (the coding HGVS already names the allele alone), so it never
    // triggers a lookup either.
    let coding_only: PubTatorDocument = serde_json::from_value(serde_json::json!({
        "pmid": 16,
        "passages": [
            {
                "infons": {"type": "abstract"},
                "text": "K-RAS c.35G>A",
                "annotations": [
                    {"text": "K-RAS", "infons": {"type": "Gene", "identifier": "3845"}},
                    {
                        "text": "c.35G>A",
                        "infons": {
                            "type": "Variant",
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
    assert_eq!(
        gene_ids_needing_official_symbols(&coding_only),
        Vec::<u64>::new()
    );
}
