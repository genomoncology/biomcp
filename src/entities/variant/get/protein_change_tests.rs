//! Sidecar tests for the gene+protein change resolution choice (ticket 1297).

use super::*;

fn protein_change_hit(
    id: &str,
    clinvar_variant_id: Option<u64>,
    rsid: Option<&str>,
) -> crate::sources::myvariant::MyVariantHit {
    serde_json::from_value(serde_json::json!({
        "_id": id,
        "dbnsfp": {"genename": "GENE", "hgvsp": "p.M1483I"},
        "dbsnp": rsid.map(|value| serde_json::json!({"rsid": value})),
        "clinvar": clinvar_variant_id.map(|value| serde_json::json!({
            "variant_id": value,
            "rcv": [{"accession": format!("RCV{value}")}]
        }))
    }))
    .expect("valid MyVariant hit")
}

#[test]
fn protein_change_resolution_prefers_the_clinvar_named_hit_over_first_match() {
    // Recorded DICER1 p.Met1483Ile shape: three alternate bases share the
    // alias and the provider ranks the ClinVar-less variant first.
    let hits = vec![
        protein_change_hit("chr14:g.95562808C>A", None, None),
        protein_change_hit("chr14:g.95562808C>T", Some(577152), Some("rs1454569806")),
        protein_change_hit("chr14:g.95562808C>G", None, None),
    ];

    let resolved = resolve_protein_change_hit("DICER1 p.Met1483Ile", "DICER1", "M1483I", hits)
        .expect("the single ClinVar-named hit resolves");
    assert_eq!(resolved.id, "chr14:g.95562808C>T");
}

#[test]
fn protein_change_resolution_keeps_a_single_matching_hit() {
    let hits = vec![protein_change_hit("chr7:g.140453136A>T", None, None)];

    let resolved = resolve_protein_change_hit("BRAF V600E", "BRAF", "V600E", hits)
        .expect("one matching hit resolves without a ClinVar record");
    assert_eq!(resolved.id, "chr7:g.140453136A>T");
}

#[test]
fn protein_change_resolution_refuses_when_no_clinvar_record_names_one() {
    // Recorded EGFR M766I shape: three alternate bases share the alias and
    // none carries a ClinVar record.
    let hits = vec![
        protein_change_hit("chr7:g.55249000G>A", None, Some("rs1322818258")),
        protein_change_hit("chr7:g.55249000G>C", None, Some("rs1322818258")),
        protein_change_hit("chr7:g.55249000G>T", None, Some("rs1322818258")),
    ];

    let error = resolve_protein_change_hit("EGFR M766I", "EGFR", "M766I", hits)
        .expect_err("no ClinVar record names one candidate");
    let BioMcpError::InvalidArgument(message) = &error else {
        panic!("ambiguity refuses as invalid argument, got: {error}");
    };
    for expected in [
        "Ambiguous protein change 'EGFR M766I'",
        "3 variants match",
        "none of them carries a ClinVar record",
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
        protein_change_hit("chr7:g.1A>T", Some(111), None),
        protein_change_hit("chr7:g.2A>T", Some(222), None),
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
