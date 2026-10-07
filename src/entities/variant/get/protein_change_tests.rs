//! Sidecar tests for the gene+protein change resolution choice (tickets
//! 1297 and 2016). Hit shapes come from the recorded MyVariant.info captures
//! named per test; each records the decisive fields only.

use super::*;

#[derive(Default)]
struct ProteinHitBuilder {
    clinvar_variant_id: Option<u64>,
    clinvar_preferred_name: Option<String>,
    rsid: Option<String>,
    snpeff_transcript: Option<String>,
    snpeff_coding: Option<String>,
    snpeff_protein: Option<String>,
}

impl ProteinHitBuilder {
    fn hit(self, id: &str, gene: &str, aliases: &str) -> crate::sources::myvariant::MyVariantHit {
        let annotation = self.snpeff_transcript.map(|feature_id| {
            serde_json::json!({
                "feature_id": feature_id,
                "genename": gene,
                "hgvs_c": self.snpeff_coding,
                "hgvs_p": self.snpeff_protein,
            })
        });
        serde_json::from_value(serde_json::json!({
            "_id": id,
            "dbnsfp": {"genename": gene, "hgvsp": aliases},
            "dbsnp": self.rsid.map(|value| serde_json::json!({"rsid": value})),
            "snpeff": annotation.map(|ann| serde_json::json!({"ann": [ann]})),
            "clinvar": self.clinvar_variant_id.map(|value| serde_json::json!({
                "variant_id": value,
                "rcv": [{
                    "accession": format!("RCV{value}"),
                    "preferred_name": self.clinvar_preferred_name,
                }]
            }))
        }))
        .expect("valid MyVariant hit")
    }
}

#[test]
fn protein_change_resolution_prefers_the_clinvar_named_hit_over_first_match() {
    // Recorded DICER1 p.Met1483Ile shape (query_dicer1_m1483i_20261006.json):
    // three alternate bases carry the alias, every transcript names
    // p.Met1483Ile, and the provider ranks the ClinVar-less variant first.
    let snpeff = |coding: &'static str| ProteinHitBuilder {
        snpeff_transcript: Some("NM_177438.2".into()),
        snpeff_coding: Some(coding.into()),
        snpeff_protein: Some("p.Met1483Ile".into()),
        ..Default::default()
    };
    let hits = vec![
        snpeff("c.4449G>T").hit("chr14:g.95562808C>A", "DICER1", "p.M1483I"),
        ProteinHitBuilder {
            clinvar_variant_id: Some(577152),
            clinvar_preferred_name: Some("NM_177438.3(DICER1):c.4449G>A (p.Met1483Ile)".into()),
            rsid: Some("rs1454569806".into()),
            snpeff_transcript: Some("NM_177438.2".into()),
            snpeff_coding: Some("c.4449G>A".into()),
            snpeff_protein: Some("p.Met1483Ile".into()),
        }
        .hit("chr14:g.95562808C>T", "DICER1", "p.M1483I"),
        snpeff("c.4449G>C").hit("chr14:g.95562808C>G", "DICER1", "p.M1483I"),
    ];

    let resolved = resolve_protein_change_hit("DICER1 p.Met1483Ile", "DICER1", "M1483I", hits)
        .expect("the ClinVar record names one of the three matching hits");
    assert_eq!(resolved.id, "chr14:g.95562808C>T");
}

#[test]
fn protein_change_resolution_keeps_the_named_change_over_a_clinvar_lookalike() {
    // Recorded TP53 C124Y shape (query_tp53_c124y_20261007.json): the
    // provider ranks the p.Cys124Tyr variant first with no ClinVar record,
    // while the ClinVar 141762 lookalike p.Cys135Tyr matches only through a
    // shorter isoform's alias.
    let hits = vec![
        ProteinHitBuilder {
            snpeff_transcript: Some("NM_000546.5".into()),
            snpeff_coding: Some("c.371G>A".into()),
            snpeff_protein: Some("p.Cys124Tyr".into()),
            ..Default::default()
        }
        .hit(
            "chr17:g.7579316C>T",
            "TP53",
            "p.Cys85Tyr, p.C124Y, p.C85Y, p.Cys124Tyr",
        ),
        ProteinHitBuilder {
            clinvar_variant_id: Some(141762),
            clinvar_preferred_name: Some("NM_000546.6(TP53):c.404G>A (p.Cys135Tyr)".into()),
            snpeff_transcript: Some("NM_000546.5".into()),
            snpeff_coding: Some("c.404G>A".into()),
            snpeff_protein: Some("p.Cys135Tyr".into()),
            ..Default::default()
        }
        .hit(
            "chr17:g.7578526C>T",
            "TP53",
            "p.Cys135Tyr, p.Cys128Tyr, p.C3Y, p.C124Y",
        ),
    ];

    let resolved = resolve_protein_change_hit("TP53 C124Y", "TP53", "C124Y", hits)
        .expect("the hit whose canonical transcript names the change resolves");
    assert_eq!(resolved.id, "chr17:g.7579316C>T");
}

#[test]
fn protein_change_resolution_keeps_the_named_brca1_change_over_a_clinvar_lookalike() {
    // Recorded BRCA1 A314T shape (query_brca1_a314t_20261007.json): the
    // canonical NM_007300.3 p.Ala314Thr hit has no ClinVar record; ClinVar
    // 55588's p.Ala1823Thr variant matches only through another isoform.
    let hits = vec![
        ProteinHitBuilder {
            snpeff_transcript: Some("NM_007300.3".into()),
            snpeff_coding: Some("c.940G>A".into()),
            snpeff_protein: Some("p.Ala314Thr".into()),
            ..Default::default()
        }
        .hit(
            "chr17:g.41246608C>T",
            "BRCA1",
            "p.Ala314Thr, p.A314T, p.Ala267Thr",
        ),
        ProteinHitBuilder {
            clinvar_variant_id: Some(55588),
            clinvar_preferred_name: Some("NM_007294.4(BRCA1):c.5467G>A (p.Ala1823Thr)".into()),
            snpeff_transcript: Some("NM_007294.3".into()),
            snpeff_coding: Some("c.5467G>A".into()),
            snpeff_protein: Some("p.Ala1823Thr".into()),
            ..Default::default()
        }
        .hit(
            "chr17:g.41199660C>T",
            "BRCA1",
            "p.Ala1823Thr, p.A314T, p.Ala719Thr",
        ),
    ];

    let resolved = resolve_protein_change_hit("BRCA1 A314T", "BRCA1", "A314T", hits)
        .expect("the hit whose canonical transcript names the change resolves");
    assert_eq!(resolved.id, "chr17:g.41246608C>T");
}

#[test]
fn protein_change_resolution_prefers_the_canonical_spelling_over_conflicting_records() {
    // Recorded BRCA1 C61G shape (query_brca1_c61g_20261007.json): both alias
    // hits carry ClinVar records, but only chr17:g.41258504A>C names C61G on
    // its canonical transcript; the 1297 conflicting-records refusal would
    // hide the unambiguous canonical answer.
    let hits = vec![
        ProteinHitBuilder {
            clinvar_variant_id: Some(409329),
            clinvar_preferred_name: Some("NM_007294.4(BRCA1):c.5482T>G (p.Cys1828Gly)".into()),
            snpeff_transcript: Some("NM_007294.3".into()),
            snpeff_coding: Some("c.5482T>G".into()),
            snpeff_protein: Some("p.Cys1828Gly".into()),
            ..Default::default()
        }
        .hit(
            "chr17:g.41197805A>C",
            "BRCA1",
            "p.Cys319Gly, p.C61G, p.Cys61Gly",
        ),
        ProteinHitBuilder {
            clinvar_variant_id: Some(17661),
            clinvar_preferred_name: Some("NM_007294.4(BRCA1):c.181T>G (p.Cys61Gly)".into()),
            snpeff_transcript: Some("NM_007294.3".into()),
            snpeff_coding: Some("c.181T>G".into()),
            snpeff_protein: Some("p.Cys61Gly".into()),
            ..Default::default()
        }
        .hit(
            "chr17:g.41258504A>C",
            "BRCA1",
            "p.Cys61Gly, p.C61G, p.Cys14Gly",
        ),
    ];

    let resolved = resolve_protein_change_hit("BRCA1 C61G", "BRCA1", "C61G", hits)
        .expect("the single canonical spelling resolves");
    assert_eq!(resolved.id, "chr17:g.41258504A>C");
}

#[test]
fn protein_change_resolution_keeps_a_single_matching_hit() {
    let hits = vec![ProteinHitBuilder::default().hit("chr7:g.140453136A>T", "GENE", "p.V600E")];

    let resolved = resolve_protein_change_hit("BRAF V600E", "BRAF", "V600E", hits)
        .expect("one matching hit resolves without a ClinVar record");
    assert_eq!(resolved.id, "chr7:g.140453136A>T");
}

#[test]
fn protein_change_resolution_refuses_when_no_clinvar_record_names_one() {
    // Recorded EGFR M766I shape (query_egfr_m766i_20261006.json): three
    // alternate bases name p.Met766Ile on NM_005228.3 and none carries a
    // ClinVar record.
    let egfr_hit = |id: &'static str, coding: &'static str| {
        ProteinHitBuilder {
            rsid: Some("rs1322818258".into()),
            snpeff_transcript: Some("NM_005228.3".into()),
            snpeff_coding: Some(coding.into()),
            snpeff_protein: Some("p.Met766Ile".into()),
            ..Default::default()
        }
        .hit(id, "EGFR", "p.Met721Ile, p.M766I, p.Met766Ile")
    };
    let hits = vec![
        egfr_hit("chr7:g.55249000G>A", "c.2298G>A"),
        egfr_hit("chr7:g.55249000G>C", "c.2298G>C"),
        egfr_hit("chr7:g.55249000G>T", "c.2298G>T"),
    ];

    let error = resolve_protein_change_hit("EGFR M766I", "EGFR", "M766I", hits)
        .expect_err("no ClinVar record names one candidate");
    let BioMcpError::InvalidArgument(message) = &error else {
        panic!("ambiguity refuses as invalid argument, got: {error}");
    };
    for expected in [
        "Ambiguous protein change 'EGFR M766I'",
        "3 variants match",
        "none of them carries a ClinVar record that names one",
        "- chr7:g.55249000G>A (rs1322818258)",
        "- chr7:g.55249000G>C (rs1322818258)",
        "- chr7:g.55249000G>T (rs1322818258)",
        "ClinVar VariationID, rsID, or a transcript-qualified HGVS",
    ] {
        assert!(
            message.contains(expected),
            "missing {expected:?} in: {message}"
        );
    }
}

#[test]
fn protein_change_resolution_refuses_conflicting_clinvar_records() {
    let hits = vec![
        ProteinHitBuilder {
            clinvar_variant_id: Some(111),
            snpeff_transcript: Some("NM_1.1".into()),
            snpeff_coding: Some("c.2T>C".into()),
            snpeff_protein: Some("p.M1I".into()),
            ..Default::default()
        }
        .hit("chr7:g.1A>T", "GENE", "p.M1I"),
        ProteinHitBuilder {
            clinvar_variant_id: Some(222),
            snpeff_transcript: Some("NM_1.1".into()),
            snpeff_coding: Some("c.2T>C".into()),
            snpeff_protein: Some("p.M1I".into()),
            ..Default::default()
        }
        .hit("chr7:g.2A>T", "GENE", "p.M1I"),
    ];

    let error = resolve_protein_change_hit("GENE M1I", "GENE", "M1I", hits)
        .expect_err("two ClinVar-named hits stay ambiguous");
    let BioMcpError::InvalidArgument(message) = &error else {
        panic!("conflicting ClinVar records refuse as invalid argument, got: {error}");
    };
    assert!(message.contains("2 of them carry conflicting ClinVar records"));
    assert!(message.contains("- chr7:g.1A>T (ClinVar VariationID 111)"));
    assert!(message.contains("- chr7:g.2A>T (ClinVar VariationID 222)"));
}

#[test]
fn protein_change_resolution_refuses_a_clinvar_lookalike_without_the_named_change() {
    // The TP53 C124Y lookalike alone: nothing names the requested change on
    // a canonical transcript, so the 1297 ClinVar tiebreak must not resolve
    // it by default.
    let hits = vec![
        ProteinHitBuilder {
            clinvar_variant_id: Some(141762),
            clinvar_preferred_name: Some("NM_000546.6(TP53):c.404G>A (p.Cys135Tyr)".into()),
            snpeff_transcript: Some("NM_000546.5".into()),
            snpeff_coding: Some("c.404G>A".into()),
            snpeff_protein: Some("p.Cys135Tyr".into()),
            ..Default::default()
        }
        .hit("chr17:g.7578526C>T", "TP53", "p.Cys135Tyr, p.C124Y"),
        ProteinHitBuilder {
            snpeff_transcript: Some("NM_001126118.1".into()),
            snpeff_coding: Some("c.287G>A".into()),
            snpeff_protein: Some("p.Cys96Tyr".into()),
            ..Default::default()
        }
        .hit("chr17:g.7578101C>T", "TP53", "p.Cys96Tyr, p.C124Y"),
    ];

    let error = resolve_protein_change_hit("TP53 C124Y", "TP53", "C124Y", hits)
        .expect_err("a lookalike must not resolve through a ClinVar record");
    let BioMcpError::InvalidArgument(message) = &error else {
        panic!("a lookalike-only match refuses as invalid argument, got: {error}");
    };
    assert!(message.contains(
        "2 variants match and none of them names that change on its canonical (or ClinVar-named) transcript"
    ));
    assert!(message.contains("- chr17:g.7578526C>T (ClinVar VariationID 141762)"));
    assert!(message.contains("- chr17:g.7578101C>T"));
}

#[test]
fn protein_change_resolution_without_matching_hits_keeps_the_search_suggestion() {
    let error = resolve_protein_change_hit("EGFR E746_A750del", "EGFR", "E746_A750del", Vec::new())
        .expect_err("no matching hit stays not found");
    let BioMcpError::NotFound { suggestion, .. } = &error else {
        panic!("a matching-less query stays not found, got: {error}");
    };
    assert!(
        suggestion.contains("biomcp search variant -g EGFR --hgvsp E746_A750del"),
        "unexpected suggestion: {suggestion}"
    );
}
