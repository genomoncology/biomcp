//! Sidecar tests for the gene+protein change resolution choice (tickets
//! 1297 and 2016). Hit shapes come from the recorded MyVariant.info captures
//! named per test; each records the decisive fields only.

use super::*;

#[derive(Default)]
struct ProteinHitBuilder {
    clinvar_variant_id: Option<u64>,
    clinvar_preferred_name: Option<String>,
    rsid: Option<String>,
    snpeff_annotations: Vec<serde_json::Value>,
}

impl ProteinHitBuilder {
    fn clinvar(mut self, variant_id: u64, preferred_name: &str) -> Self {
        self.clinvar_variant_id = Some(variant_id);
        self.clinvar_preferred_name = Some(preferred_name.into());
        self
    }

    fn rsid(mut self, rsid: &str) -> Self {
        self.rsid = Some(rsid.into());
        self
    }

    fn with_snpeff(mut self, feature_id: &str, coding: &str, protein: Option<&str>) -> Self {
        self.snpeff_annotations.push(serde_json::json!({
            "feature_id": feature_id,
            "hgvs_c": coding,
            "hgvs_p": protein,
        }));
        self
    }

    fn hit(self, id: &str, gene: &str, aliases: &str) -> crate::sources::myvariant::MyVariantHit {
        let annotations = self.snpeff_annotations.clone();
        serde_json::from_value(serde_json::json!({
            "_id": id,
            "dbnsfp": {"genename": gene, "hgvsp": aliases},
            "dbsnp": self.rsid.map(|value| serde_json::json!({"rsid": value})),
            "snpeff": (!annotations.is_empty()).then(|| serde_json::json!({"ann": annotations})),
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

/// The full recorded SnpEff annotation list of the ClinVar-less BRCA1 A314T
/// hit (query_brca1_a314t_20261007.json): NM_007300.3 leads the list, so the
/// first-NM_ rule headlines the long isoform, while NM_007294 (MANE Select)
/// also names p.Ala314Thr.
fn brca1_a314t_snpeff(builder: ProteinHitBuilder) -> ProteinHitBuilder {
    builder
        .with_snpeff("NM_007300.3", "c.940G>A", Some("p.Ala314Thr"))
        .with_snpeff("NM_007297.3", "c.799G>A", Some("p.Ala267Thr"))
        .with_snpeff("NM_007294.3", "c.940G>A", Some("p.Ala314Thr"))
        .with_snpeff("NM_007298.3", "c.787+153G>A", None)
        .with_snpeff("NR_027676.1", "n.1076G>A", None)
}

/// The full recorded SnpEff annotation list of the ClinVar-less BRCA1 I1568N
/// hit (query_brca1_i1568n_20261007.json), past the isoform insert where
/// NM_007300 numbers the residue 21 higher than NM_007294 (MANE Select).
fn brca1_i1568n_snpeff(builder: ProteinHitBuilder) -> ProteinHitBuilder {
    builder
        .with_snpeff("NM_007300.3", "c.4766T>A", Some("p.Ile1589Asn"))
        .with_snpeff("NM_007294.3", "c.4703T>A", Some("p.Ile1568Asn"))
}

/// The ClinVar-marked lookalike of the recorded BRCA1 A314T query
/// (query_brca1_a314t_20261007.json): ClinVar 55588's preferred names sit on
/// NM_007294.4, the gene's MANE Select transcript.
fn brca1_clinvar_55588_hit() -> crate::sources::myvariant::MyVariantHit {
    ProteinHitBuilder::default()
        .clinvar(55588, "NM_007294.4(BRCA1):c.5467G>A (p.Ala1823Thr)")
        .with_snpeff("NM_007300.3", "c.5530G>A", Some("p.Ala1844Thr"))
        .with_snpeff("NM_007294.3", "c.5467G>A", Some("p.Ala1823Thr"))
        .hit(
            "chr17:g.41199660C>T",
            "BRCA1",
            "p.Ala1823Thr, p.A314T, p.Ala719Thr, p.I1568N",
        )
}

#[test]
fn protein_change_resolution_prefers_the_clinvar_named_hit_over_first_match() {
    // Recorded DICER1 p.Met1483Ile shape (query_dicer1_m1483i_20261006.json):
    // three alternate bases carry the alias, every transcript names
    // p.Met1483Ile, and the provider ranks the ClinVar-less variant first.
    let snpeff = |coding: &'static str| {
        ProteinHitBuilder::default().with_snpeff("NM_177438.2", coding, Some("p.Met1483Ile"))
    };
    let hits = vec![
        snpeff("c.4449G>T").hit("chr14:g.95562808C>A", "DICER1", "p.M1483I"),
        ProteinHitBuilder::default()
            .clinvar(577152, "NM_177438.3(DICER1):c.4449G>A (p.Met1483Ile)")
            .rsid("rs1454569806")
            .with_snpeff("NM_177438.2", "c.4449G>A", Some("p.Met1483Ile"))
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
        ProteinHitBuilder::default()
            .with_snpeff("NM_000546.5", "c.371G>A", Some("p.Cys124Tyr"))
            .hit(
                "chr17:g.7579316C>T",
                "TP53",
                "p.Cys85Tyr, p.C124Y, p.C85Y, p.Cys124Tyr",
            ),
        ProteinHitBuilder::default()
            .clinvar(141762, "NM_000546.6(TP53):c.404G>A (p.Cys135Tyr)")
            .with_snpeff("NM_000546.5", "c.404G>A", Some("p.Cys135Tyr"))
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
    // p.Ala314Thr hit has no ClinVar record; ClinVar 55588's p.Ala1823Thr
    // variant matches only through another isoform.
    let hits = vec![
        brca1_a314t_snpeff(ProteinHitBuilder::default()).hit(
            "chr17:g.41246608C>T",
            "BRCA1",
            "p.Ala314Thr, p.A314T, p.Ala267Thr",
        ),
        brca1_clinvar_55588_hit(),
    ];

    let resolved = resolve_protein_change_hit("BRCA1 A314T", "BRCA1", "A314T", hits)
        .expect("the hit whose canonical transcript names the change resolves");
    assert_eq!(resolved.id, "chr17:g.41246608C>T");
}

#[test]
fn protein_change_resolution_headlines_the_mane_transcript_of_the_query() {
    // The same recorded BRCA1 A314T cohort: the lookalike's ClinVar preferred
    // names sit on NM_007294, the gene's MANE Select transcript, so the
    // ClinVar-less hit headlines NM_007294 instead of the first NM_ in the
    // SnpEff list (NM_007300.3, which numbers residues 21 higher past the
    // isoform insert). The marker carries ClinVar's current accession
    // version (.4) where the SnpEff build lags (.3), so the headline shows
    // the current accession (ticket 2035 finding 21).
    let hits = vec![
        brca1_a314t_snpeff(ProteinHitBuilder::default()).hit(
            "chr17:g.41246608C>T",
            "BRCA1",
            "p.Ala314Thr, p.A314T, p.Ala267Thr",
        ),
        brca1_clinvar_55588_hit(),
    ];
    let mane = transform::variant::clinvar_mane_transcript(&hits);
    assert_eq!(mane.as_deref(), Some("NM_007294.4"));

    let resolved = resolve_protein_change_hit("BRCA1 A314T", "BRCA1", "A314T", hits)
        .expect("the named hit resolves");
    let variant =
        transform::variant::from_myvariant_hit_with_mane(&resolved, mane.as_deref(), Some("A314T"));
    assert_eq!(variant.hgvs_p.as_deref(), Some("p.Ala314Thr"));
    assert_eq!(variant.transcript.as_deref(), Some("NM_007294.4"));
    assert_eq!(variant.hgvs_c.as_deref(), Some("c.940G>A"));
}

#[test]
fn protein_change_resolution_resolves_a_past_insert_change_on_mane_numbering() {
    // Recorded BRCA1 I1568N hit (query_brca1_i1568n_20261007.json) composed
    // with the recorded ClinVar 55588 lookalike (query_brca1_a314t_20261007
    // .json): past the isoform insert NM_007300 numbers the residue 21
    // higher than MANE Select NM_007294 (p.Ile1589Asn vs p.Ile1568Asn).
    // With the query's ClinVar-preferred stem marking NM_007294, the true
    // hit names the requested I1568N and resolves; the first-NM_ rule would
    // headline p.Ile1589Asn, leave the request unnamed, and hand the query
    // to the lookalike's alias match.
    let hits = vec![
        brca1_i1568n_snpeff(ProteinHitBuilder::default()).hit(
            "chr17:g.41223228A>T",
            "BRCA1",
            "p.Ile1589Asn, p.I1568N, p.I1589N",
        ),
        brca1_clinvar_55588_hit(),
    ];
    let mane = transform::variant::clinvar_mane_transcript(&hits);
    assert_eq!(mane.as_deref(), Some("NM_007294.4"));

    let resolved = resolve_protein_change_hit("BRCA1 I1568N", "BRCA1", "I1568N", hits)
        .expect("the MANE-named hit resolves past the insert");
    assert_eq!(resolved.id, "chr17:g.41223228A>T");
    let variant = transform::variant::from_myvariant_hit_with_mane(
        &resolved,
        mane.as_deref(),
        Some("I1568N"),
    );
    assert_eq!(variant.hgvs_p.as_deref(), Some("p.Ile1568Asn"));
    assert_eq!(variant.transcript.as_deref(), Some("NM_007294.4"));
    assert_eq!(variant.protein_numbering_note, None);
}

#[test]
fn protein_change_resolution_resolves_the_naming_annotation_without_a_marker() {
    // The same recorded I1568N cohort with the lookalike carrying no ClinVar
    // preferred names: nothing in the response marks a MANE transcript, so
    // the annotation that names the request is the signal (ticket 2035
    // finding 4) — the true hit headlines NM_007294's p.Ile1568Asn and
    // resolves instead of refusing while the data names the change.
    let hits = vec![
        brca1_i1568n_snpeff(ProteinHitBuilder::default()).hit(
            "chr17:g.41223228A>T",
            "BRCA1",
            "p.Ile1589Asn, p.I1568N, p.I1589N",
        ),
        ProteinHitBuilder::default()
            .with_snpeff("NM_007294.3", "c.5467G>A", Some("p.Ala1823Thr"))
            .hit(
                "chr17:g.41199660C>T",
                "BRCA1",
                "p.Ala1823Thr, p.I1568N, p.Ala719Thr",
            ),
    ];
    assert_eq!(transform::variant::clinvar_mane_transcript(&hits), None);

    let resolved = resolve_protein_change_hit("BRCA1 I1568N", "BRCA1", "I1568N", hits)
        .expect("the naming annotation resolves without a marker");
    assert_eq!(resolved.id, "chr17:g.41223228A>T");
    let variant = transform::variant::from_myvariant_hit_with_mane(&resolved, None, Some("I1568N"));
    assert_eq!(variant.hgvs_p.as_deref(), Some("p.Ile1568Asn"));
    assert_eq!(variant.transcript.as_deref(), Some("NM_007294.3"));
    assert_eq!(variant.protein_numbering_note, None);
}

#[test]
fn protein_change_resolution_prefers_the_canonical_spelling_over_conflicting_records() {
    // Recorded BRCA1 C61G shape (query_brca1_c61g_20261007.json): both alias
    // hits carry ClinVar records, but only chr17:g.41258504A>C names C61G on
    // its canonical transcript; the 1297 conflicting-records refusal would
    // hide the unambiguous canonical answer.
    let hits = vec![
        ProteinHitBuilder::default()
            .clinvar(409329, "NM_007294.4(BRCA1):c.5482T>G (p.Cys1828Gly)")
            .with_snpeff("NM_007294.3", "c.5482T>G", Some("p.Cys1828Gly"))
            .hit(
                "chr17:g.41197805A>C",
                "BRCA1",
                "p.Cys319Gly, p.C61G, p.Cys61Gly",
            ),
        ProteinHitBuilder::default()
            .clinvar(17661, "NM_007294.4(BRCA1):c.181T>G (p.Cys61Gly)")
            .with_snpeff("NM_007294.3", "c.181T>G", Some("p.Cys61Gly"))
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
    let hits = vec![
        ProteinHitBuilder::default()
            .with_snpeff("NM_1.1", "c.1T>A", Some("p.V600E"))
            .hit("chr7:g.140453136A>T", "GENE", "p.V600E"),
    ];

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
        ProteinHitBuilder::default()
            .rsid("rs1322818258")
            .with_snpeff("NM_005228.3", coding, Some("p.Met766Ile"))
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
        ProteinHitBuilder::default()
            .clinvar(111, "NM_1.1(GENE):c.2T>C (p.M1I)")
            .with_snpeff("NM_1.1", "c.2T>C", Some("p.M1I"))
            .hit("chr7:g.1A>T", "GENE", "p.M1I"),
        ProteinHitBuilder::default()
            .clinvar(222, "NM_1.1(GENE):c.2T>C (p.M1I)")
            .with_snpeff("NM_1.1", "c.2T>C", Some("p.M1I"))
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
        ProteinHitBuilder::default()
            .clinvar(141762, "NM_000546.6(TP53):c.404G>A (p.Cys135Tyr)")
            .with_snpeff("NM_000546.5", "c.404G>A", Some("p.Cys135Tyr"))
            .hit("chr17:g.7578526C>T", "TP53", "p.Cys135Tyr, p.C124Y"),
        ProteinHitBuilder::default()
            .with_snpeff("NM_001126118.1", "c.287G>A", Some("p.Cys96Tyr"))
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

#[test]
fn protein_change_resolution_notes_the_numbering_when_the_hit_does_not_spell_the_request() {
    // Recorded TP53 R116Q shape (query_tp53_r116q_20261007.json): the single
    // hit is ClinVar 12356, whose preferred name sits on NM_000546, TP53's
    // MANE Select transcript; the request follows a shorter isoform's
    // numbering (p.Arg116Gln), so the answer must say which numbering
    // matched instead of resolving silently. ClinVar's accession version
    // (.6) is current where the SnpEff build lags (.5), so the note names
    // the current accession (ticket 2035 finding 21).
    let hits = vec![
        ProteinHitBuilder::default()
            .clinvar(12356, "NM_000546.6(TP53):c.743G>A (p.Arg248Gln)")
            .with_snpeff("NM_000546.5", "c.743G>A", Some("p.Arg248Gln"))
            .hit(
                "chr17:g.7577538C>T",
                "TP53",
                "p.Arg248Gln, p.R116Q, p.R248Q",
            ),
    ];

    let resolved = resolve_protein_change_hit("TP53 R116Q", "TP53", "R116Q", hits)
        .expect("a unique provider hit resolves");
    let mane = transform::variant::clinvar_mane_transcript(std::slice::from_ref(&resolved));
    assert_eq!(mane.as_deref(), Some("NM_000546.6"));
    let note = protein_change_numbering_note(&resolved, "R116Q", mane.as_deref())
        .expect("the answer names the numbering that matched");
    assert_eq!(
        note,
        "Numbering note: resolved on NM_000546.6 as p.Arg248Gln; the \
         requested R116Q follows another transcript's numbering."
    );

    let variant =
        transform::variant::from_myvariant_hit_with_mane(&resolved, mane.as_deref(), Some("R116Q"));
    assert_eq!(variant.hgvs_p.as_deref(), Some("p.Arg248Gln"));
    assert_eq!(variant.transcript.as_deref(), Some("NM_000546.6"));
}

#[test]
fn protein_change_resolution_resolves_the_requested_numbering_without_a_marker() {
    // Recorded BRCA1 I1568N shape (query_brca1_i1568n_20261007.json): the
    // single ClinVar-less response carries no MANE marker, and the
    // annotation naming the request is the signal (ticket 2035 finding 4)
    // — NM_007294's p.Ile1568Asn spells the request, so the answer resolves
    // on MANE Select numbering with no note instead of answering p.Ile1589Asn
    // on the first NM_ (NM_007300.3) with a note claiming the request
    // follows another transcript's numbering.
    let hits = vec![brca1_i1568n_snpeff(ProteinHitBuilder::default()).hit(
        "chr17:g.41223228A>T",
        "BRCA1",
        "p.Ile1589Asn, p.I1568N, p.I1589N",
    )];

    let resolved = resolve_protein_change_hit("BRCA1 I1568N", "BRCA1", "I1568N", hits)
        .expect("a unique provider hit resolves");
    let mane = transform::variant::clinvar_mane_transcript(std::slice::from_ref(&resolved));
    assert_eq!(mane, None);
    assert_eq!(
        protein_change_numbering_note(&resolved, "I1568N", None),
        None,
        "the headline spells the request, so no numbering note prints"
    );
    let variant = transform::variant::from_myvariant_hit_with_mane(
        &resolved,
        mane.as_deref(),
        Some("I1568N"),
    );
    assert_eq!(variant.hgvs_p.as_deref(), Some("p.Ile1568Asn"));
    assert_eq!(variant.transcript.as_deref(), Some("NM_007294.3"));
    assert_eq!(variant.protein_numbering_note, None);
}

#[test]
fn protein_change_resolution_skips_the_note_when_the_hit_spells_the_request() {
    let hits = vec![brca1_a314t_snpeff(ProteinHitBuilder::default()).hit(
        "chr17:g.41246608C>T",
        "BRCA1",
        "p.Ala314Thr, p.A314T, p.Ala267Thr",
    )];

    let resolved = resolve_protein_change_hit("BRCA1 A314T", "BRCA1", "A314T", hits)
        .expect("a unique provider hit resolves");
    assert_eq!(
        protein_change_numbering_note(&resolved, "A314T", None),
        None
    );
}

#[test]
fn requested_reference_residue_parses_compact_substitutions() {
    assert_eq!(requested_reference_residue("R209Q"), Some(('R', 209)));
    assert_eq!(requested_reference_residue("p.Gly112Asp"), Some(('G', 112)));
    assert_eq!(requested_reference_residue("E746_A750del"), None);
    assert_eq!(requested_reference_residue("600E"), None);
}

#[test]
fn mane_annotation_names_change_reads_the_mane_stem_only() {
    // Recorded BRCA1 I1568N shape (query_brca1_i1568n_20261007.json): the
    // hit's NM_007294 annotation names I1568N even though the first-NM_
    // headline picks NM_007300's p.Ile1589Asn. The marker may carry a
    // versioned accession (ticket 2035 finding 21); the stem is what
    // matches.
    let hit = brca1_i1568n_snpeff(ProteinHitBuilder::default()).hit(
        "chr17:g.41223228A>T",
        "BRCA1",
        "p.Ile1589Asn, p.I1568N, p.I1589N",
    );
    assert!(transform::variant::mane_annotation_names_change(
        &hit,
        "I1568N",
        Some("NM_007294")
    ));
    assert!(transform::variant::mane_annotation_names_change(
        &hit,
        "I1568N",
        Some("NM_007294.4")
    ));
    assert!(!transform::variant::mane_annotation_names_change(
        &hit,
        "I1568N",
        Some("NM_007300")
    ));
    // No known MANE transcript: nothing can be named on one.
    assert!(!transform::variant::mane_annotation_names_change(
        &hit, "I1568N", None
    ));

    // Recorded TP53 R209Q shape (query_tp53_r209q_20261008.json): the hit
    // names p.Arg248Gln on NM_000546 and never R209Q there. The response's
    // ClinVar marker must outrank the annotation that names the request on
    // the shorter isoform, or the lookalike would resolve through it
    // (ticket 2033 finding 3; ticket 2035 finding 4).
    let lookalike = ProteinHitBuilder::default()
        .clinvar(12356, "NM_000546.6(TP53):c.743G>A (p.Arg248Gln)")
        .with_snpeff("NM_000546.5", "c.743G>A", Some("p.Arg248Gln"))
        .with_snpeff("NM_001126118.1", "c.626G>A", Some("p.Arg209Gln"))
        .hit(
            "chr17:g.7577538C>T",
            "TP53",
            "p.Arg248Gln, p.R209Q, p.R248Q",
        );
    assert!(!transform::variant::mane_annotation_names_change(
        &lookalike,
        "R209Q",
        Some("NM_000546")
    ));
    let variant = transform::variant::from_myvariant_hit_with_mane(
        &lookalike,
        Some("NM_000546.6"),
        Some("R209Q"),
    );
    assert_eq!(variant.hgvs_p.as_deref(), Some("p.Arg248Gln"));
    assert_eq!(variant.transcript.as_deref(), Some("NM_000546.6"));
}

#[test]
fn headline_shows_the_current_transcript_version_clinvar_carries() {
    // Recorded BRAF V600E shape (get_braf_v600e_grch38_20260806.json): the
    // hit's own ClinVar preferred names carry NM_004333.6 while the SnpEff
    // build still says NM_004333.4, so the answer headlines the current
    // MANE accession (ticket 2035 finding 21).
    let hit: crate::sources::myvariant::MyVariantHit = serde_json::from_value(serde_json::json!({
        "_id": "chr7:g.140453136A>T",
        "dbnsfp": {"genename": "BRAF", "hgvsp": "p.Val600Glu"},
        "clinvar": {"variant_id": 13961, "rcv": [{
            "preferred_name": "NM_004333.6(BRAF):c.1799T>A (p.Val600Glu)",
        }]},
        "snpeff": {"ann": [{
            "feature_id": "NM_004333.4",
            "genename": "BRAF",
            "hgvs_c": "c.1799T>A",
            "hgvs_p": "p.Val600Glu",
        }]}
    }))
    .expect("valid MyVariant hit");
    let variant = transform::variant::from_myvariant_hit(&hit);
    assert_eq!(variant.hgvs_p.as_deref(), Some("p.Val600Glu"));
    assert_eq!(variant.hgvs_c.as_deref(), Some("c.1799T>A"));
    assert_eq!(variant.transcript.as_deref(), Some("NM_004333.6"));
}

#[test]
fn mane_numbering_refusal_names_the_residue_and_the_lookalike() {
    // Recorded TP53 R209Q shape: UniProt P04637 (the canonical/MANE Select
    // protein) has Arg at 209, so the request's numbering is valid on MANE
    // while the only alias match names p.Arg248Gln — a different change.
    // The refusal must say both facts instead of the other-transcript note,
    // and its retry line names the requested change, because the candidate
    // is already known to be a different change (ticket 2035 finding 21).
    let error = mane_numbering_refusal_message(
        "TP53 R209Q",
        "TP53",
        "R209Q",
        "P04637",
        "Arg",
        209,
        "p.Arg248Gln",
        "NM_000546.6",
        "chr17:g.7577538C>T (ClinVar VariationID 12356)",
    );
    let BioMcpError::InvalidArgument(message) = &error else {
        panic!("a MANE-numbered request refuses as invalid argument, got: {error}");
    };
    for expected in [
        "No MANE-numbered variant matches 'TP53 R209Q'",
        "canonical protein (UniProt P04637) has Arg at 209",
        "no matching record names that change",
        "the only alias match is p.Arg248Gln on NM_000546.6",
        "- chr17:g.7577538C>T (ClinVar VariationID 12356)",
        "Retry `biomcp get variant` with a transcript-qualified HGVS naming 'R209Q'",
        "biomcp search variant -g TP53 --hgvsp R209Q",
    ] {
        assert!(
            message.contains(expected),
            "missing {expected:?} in: {message}"
        );
    }
    assert!(
        !message.contains("follows another transcript's numbering"),
        "the refusal must not carry the other-transcript note: {message}"
    );
    assert!(
        !message.contains("one candidate's exact form"),
        "the retry line must not point at the known-differing candidate: {message}"
    );
}
