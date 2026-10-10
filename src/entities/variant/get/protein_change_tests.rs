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

    /// A SnpEff protein-structure row (`feature_type: interaction`, a PDB
    /// chain feature naming no transcript of its own — ticket 2042).
    fn with_interaction(mut self, feature_id: &str, coding: &str) -> Self {
        self.snpeff_annotations.push(serde_json::json!({
            "feature_id": feature_id,
            "feature_type": "interaction",
            "hgvs_c": coding,
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

    let mane = transform::variant::clinvar_mane_transcript(&hits);
    let resolved = resolve_protein_change_hit(
        "DICER1 p.Met1483Ile",
        "DICER1",
        "M1483I",
        hits,
        mane.as_deref(),
    )
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

    let mane = transform::variant::clinvar_mane_transcript(&hits);
    let resolved = resolve_protein_change_hit("TP53 C124Y", "TP53", "C124Y", hits, mane.as_deref())
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

    let mane = transform::variant::clinvar_mane_transcript(&hits);
    let resolved =
        resolve_protein_change_hit("BRCA1 A314T", "BRCA1", "A314T", hits, mane.as_deref())
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

    let resolved =
        resolve_protein_change_hit("BRCA1 A314T", "BRCA1", "A314T", hits, mane.as_deref())
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

    let resolved =
        resolve_protein_change_hit("BRCA1 I1568N", "BRCA1", "I1568N", hits, mane.as_deref())
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

    let resolved = resolve_protein_change_hit("BRCA1 I1568N", "BRCA1", "I1568N", hits, None)
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

    let mane = transform::variant::clinvar_mane_transcript(&hits);
    let resolved = resolve_protein_change_hit("BRCA1 C61G", "BRCA1", "C61G", hits, mane.as_deref())
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

    let resolved = resolve_protein_change_hit("BRAF V600E", "BRAF", "V600E", hits, None)
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

    let error = resolve_protein_change_hit("EGFR M766I", "EGFR", "M766I", hits, None)
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

    let error = resolve_protein_change_hit("GENE M1I", "GENE", "M1I", hits, None)
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

    let error = resolve_protein_change_hit("TP53 C124Y", "TP53", "C124Y", hits, None)
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
    let error = resolve_protein_change_hit(
        "EGFR E746_A750del",
        "EGFR",
        "E746_A750del",
        Vec::new(),
        None,
    )
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

    let mane = transform::variant::clinvar_mane_transcript(&hits);
    let resolved = resolve_protein_change_hit("TP53 R116Q", "TP53", "R116Q", hits, mane.as_deref())
        .expect("a unique provider hit resolves");
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

    let resolved = resolve_protein_change_hit("BRCA1 I1568N", "BRCA1", "I1568N", hits, None)
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

/// Canonical-protein facts as the committed UniProt captures carry them:
/// the accession, the residue at the requested position (None when the
/// position lies past the sequence's end), the sequence's length, and the
/// record's MANE-Select cross-reference (tickets 2036 and 2042).
fn canonical_facts(
    accession: &str,
    residue: Option<char>,
    sequence_length: u32,
    mane: &str,
) -> CanonicalProteinFacts {
    CanonicalProteinFacts {
        accession: accession.into(),
        residue,
        sequence_length: Some(sequence_length),
        mane_transcript: Some(mane.into()),
    }
}

/// Recorded TP53 S183Y shape (query_tp53_s183y_20261009.json): the single
/// ClinVar-less hit spells the request only on shorter isoforms
/// (NM_001126115.1 p.Ser183Tyr) while MANE Select NM_000546 names
/// p.Ser315Tyr. UniProt P04637 (the canonical/MANE Select protein, kept in
/// get_p04637_20261008.json) has Ser at 183, so the requested numbering is
/// valid on MANE while the only alias match names a different change: the
/// canonical-residue check must run before the isoform annotation is
/// accepted, and the answer refuses instead of resolving `p.Ser183Tyr` on
/// NM_001126115.1 with no note (ticket 2036).
#[test]
fn protein_change_resolution_refuses_an_isoform_spelling_of_a_mane_numbered_request() {
    let hits = vec![
        ProteinHitBuilder::default()
            .rsid("rs2073157445")
            .with_snpeff("NM_000546.5", "c.944C>A", Some("p.Ser315Tyr"))
            .with_snpeff("NM_001126115.1", "c.548C>A", Some("p.Ser183Tyr"))
            .with_snpeff("NM_001276697.1", "c.467C>A", Some("p.Ser156Tyr"))
            .hit(
                "chr17:g.7576902G>T",
                "TP53",
                "p.S183Y, p.S156Y, p.S315Y, p.Ser315Tyr, p.Ser183Tyr",
            ),
    ];

    let facts = canonical_facts("P04637", Some('S'), 393, "NM_000546.6");
    let resolved = resolve_protein_change_hit(
        "TP53 S183Y",
        "TP53",
        "S183Y",
        hits,
        facts.mane_transcript.as_deref(),
    )
    .expect("a unique provider hit resolves before the numbering check");
    let error = refuse_or_note_with_facts(
        "TP53 S183Y",
        "TP53",
        "S183Y",
        &resolved,
        facts.mane_transcript.as_deref(),
        &facts,
    )
    .expect_err("an isoform-only spelling of a MANE-numbered request refuses");
    let BioMcpError::InvalidArgument(message) = &error else {
        panic!("the isoform lookalike refuses as invalid argument, got: {error}");
    };
    for expected in [
        "No MANE-numbered variant matches 'TP53 S183Y'",
        "canonical protein (UniProt P04637) has Ser at 183",
        "the only alias match is p.Ser315Tyr on NM_000546.6",
        "the request is spelled p.Ser183Tyr on NM_001126115.1, another transcript",
        "Retry `biomcp get variant` with a transcript-qualified HGVS naming 'S183Y'",
    ] {
        assert!(
            message.contains(expected),
            "missing {expected:?} in: {message}"
        );
    }
    assert!(
        !message.contains("Candidates:"),
        "the refusal names the isoform instead of a bare candidate list: {message}"
    );
    assert!(
        !message.contains("follows another transcript's numbering"),
        "the refusal must not claim the request follows another numbering: {message}"
    );
}

/// Recorded BRCA1 S1587F shape (query_brca1_s1587f_20261009.json): the
/// single ClinVar-less hit's first-NM_ annotation is the long isoform's
/// `p.Ser1587Phe` on NM_007300.3 while MANE Select NM_007294 names
/// `p.Ser1566Phe`. UniProt P38398 has Ser at 1587 as well as at 1566, so
/// the request's numbering is valid on the canonical protein and no record
/// names that change on MANE: the answer refuses, naming the MANE spelling
/// (ticket 2036), instead of resolving `p.Ser1587Phe` on NM_007300.3.
#[test]
fn protein_change_resolution_refuses_a_long_isoform_spelling_of_a_mane_numbered_request() {
    let hits = vec![
        ProteinHitBuilder::default()
            .rsid("rs1060502325")
            .with_snpeff("NM_007300.3", "c.4760C>T", Some("p.Ser1587Phe"))
            .with_snpeff("NM_007298.3", "c.1385C>T", Some("p.Ser462Phe"))
            .with_snpeff("NM_007297.3", "c.4556C>T", Some("p.Ser1519Phe"))
            .with_snpeff("NM_007294.3", "c.4697C>T", Some("p.Ser1566Phe"))
            .hit(
                "chr17:g.41223234G>A",
                "BRCA1",
                "p.Ser1587Phe, p.S1587F, p.Ser1566Phe, p.S1566F",
            ),
    ];

    let facts = canonical_facts("P38398", Some('S'), 1863, "NM_007294.4");
    let resolved = resolve_protein_change_hit(
        "BRCA1 S1587F",
        "BRCA1",
        "S1587F",
        hits,
        facts.mane_transcript.as_deref(),
    )
    .expect("a unique provider hit resolves before the numbering check");
    let error = refuse_or_note_with_facts(
        "BRCA1 S1587F",
        "BRCA1",
        "S1587F",
        &resolved,
        facts.mane_transcript.as_deref(),
        &facts,
    )
    .expect_err("an isoform-only spelling of a MANE-numbered request refuses");
    let BioMcpError::InvalidArgument(message) = &error else {
        panic!("the isoform lookalike refuses as invalid argument, got: {error}");
    };
    assert!(message.contains("canonical protein (UniProt P38398) has Ser at 1587"));
    assert!(message.contains("the only alias match is p.Ser1566Phe on NM_007294.4"));
    assert!(
        message.contains("the request is spelled p.Ser1587Phe on NM_007300.3, another transcript"),
        "the refusal names the long isoform that spells the request: {message}"
    );
    assert!(!message.contains("Candidates:"), "{message}");
}

/// Recorded BRCA1 S1551Y and S395T shapes (query_brca1_s1551y_20261009.json,
/// query_brca1_s395t_20261009.json): two ClinVar-less alias hits apiece, and
/// exactly one of them names the change on MANE Select NM_007294 (the other
/// matches through another isoform's spelling). The MANE marker from the
/// canonical record's MANE-Select cross-reference resolves the MANE-named
/// candidate instead of refusing, and the headline shows the current
/// accession version (ticket 2036).
#[test]
fn protein_change_resolution_returns_the_mane_named_candidate_of_two_alias_hits() {
    let s1551y_hits = vec![
        ProteinHitBuilder::default()
            .rsid("rs2153837843")
            .with_snpeff("NM_007300.3", "c.4856C>A", Some("p.Ser1619Tyr"))
            .with_snpeff("NM_007297.3", "c.4652C>A", Some("p.Ser1551Tyr"))
            .with_snpeff("NM_007294.3", "c.4793C>A", Some("p.Ser1598Tyr"))
            .hit(
                "chr17:g.41223138G>T",
                "BRCA1",
                "p.Ser1551Tyr, p.S1551Y, p.Ser1598Tyr",
            ),
        ProteinHitBuilder::default()
            .rsid("rs2052590804")
            .with_snpeff("NM_007300.3", "c.4715C>A", Some("p.Ser1572Tyr"))
            .with_snpeff("NM_007298.3", "c.1340C>A", Some("p.Ser447Tyr"))
            .with_snpeff("NM_007294.3", "c.4652C>A", Some("p.Ser1551Tyr"))
            .hit(
                "chr17:g.41226371G>T",
                "BRCA1",
                "p.Ser1551Tyr, p.S1551Y, p.Ser1572Tyr",
            ),
    ];
    let resolved = resolve_protein_change_hit(
        "BRCA1 S1551Y",
        "BRCA1",
        "S1551Y",
        s1551y_hits,
        Some("NM_007294.4"),
    )
    .expect("the candidate MANE names resolves");
    assert_eq!(resolved.id, "chr17:g.41226371G>T");
    let variant = transform::variant::from_myvariant_hit_with_mane(
        &resolved,
        Some("NM_007294.4"),
        Some("S1551Y"),
    );
    assert_eq!(variant.hgvs_p.as_deref(), Some("p.Ser1551Tyr"));
    assert_eq!(variant.transcript.as_deref(), Some("NM_007294.4"));
    assert_eq!(variant.protein_numbering_note, None);

    let s395t_hits = vec![
        ProteinHitBuilder::default()
            .rsid("rs2154023734")
            .with_snpeff("NM_007300.3", "c.4558T>A", Some("p.Ser1520Thr"))
            .with_snpeff("NM_007298.3", "c.1183T>A", Some("p.Ser395Thr"))
            .with_snpeff("NM_007294.3", "c.4495T>A", Some("p.Ser1499Thr"))
            .hit(
                "chr17:g.41226528A>T",
                "BRCA1",
                "p.Ser395Thr, p.S395T, p.Ser1499Thr",
            ),
        ProteinHitBuilder::default()
            .rsid("rs2154471807")
            .with_snpeff("NM_007300.3", "c.1183T>A", Some("p.Ser395Thr"))
            .with_snpeff("NM_007297.3", "c.1042T>A", Some("p.Ser348Thr"))
            .with_snpeff("NM_007294.3", "c.1183T>A", Some("p.Ser395Thr"))
            .hit(
                "chr17:g.41246365A>T",
                "BRCA1",
                "p.Ser395Thr, p.S395T, p.Ser348Thr",
            ),
    ];
    let resolved = resolve_protein_change_hit(
        "BRCA1 S395T",
        "BRCA1",
        "S395T",
        s395t_hits,
        Some("NM_007294.4"),
    )
    .expect("the candidate MANE names resolves");
    assert_eq!(resolved.id, "chr17:g.41246365A>T");
    let variant = transform::variant::from_myvariant_hit_with_mane(
        &resolved,
        Some("NM_007294.4"),
        Some("S395T"),
    );
    assert_eq!(variant.hgvs_p.as_deref(), Some("p.Ser395Thr"));
    assert_eq!(variant.transcript.as_deref(), Some("NM_007294.4"));
    assert_eq!(variant.protein_numbering_note, None);
}

/// The honest note is pinned through the same decision the flow runs once
/// the canonical facts are in hand: UniProt P04637 has Ser at 116, not Arg,
/// so the requested R116Q truly follows another transcript's numbering and
/// the answer must say which numbering matched. Silencing the note (or
/// swapping it for a refusal) fails this test (ticket 2036 pins the 2016
/// behavior).
#[test]
fn protein_change_resolution_pins_the_true_other_transcript_numbering_note() {
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
    let mane = transform::variant::clinvar_mane_transcript(&hits);
    let resolved = resolve_protein_change_hit("TP53 R116Q", "TP53", "R116Q", hits, mane.as_deref())
        .expect("a unique provider hit resolves");

    let facts = canonical_facts("P04637", Some('S'), 393, "NM_000546.6");
    let note = refuse_or_note_with_facts(
        "TP53 R116Q",
        "TP53",
        "R116Q",
        &resolved,
        mane.as_deref(),
        &facts,
    )
    .expect("the true other-transcript numbering prints its note");
    assert_eq!(
        note,
        Some(
            "Numbering note: resolved on NM_000546.6 as p.Arg248Gln; the \
             requested R116Q follows another transcript's numbering."
                .to_string(),
        )
    );
}

#[test]
fn protein_change_resolution_skips_the_note_when_the_hit_spells_the_request() {
    let hits = vec![brca1_a314t_snpeff(ProteinHitBuilder::default()).hit(
        "chr17:g.41246608C>T",
        "BRCA1",
        "p.Ala314Thr, p.A314T, p.Ala267Thr",
    )];

    let resolved = resolve_protein_change_hit("BRCA1 A314T", "BRCA1", "A314T", hits, None)
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
    // The recorded R209Q record: the ClinVar 12356 hit whose only
    // annotation names p.Arg248Gln on MANE and holds no row spelling
    // R209Q, so the refusal carries no isoform clause either.
    let hit = ProteinHitBuilder::default()
        .clinvar(12356, "NM_000546.6(TP53):c.743G>A (p.Arg248Gln)")
        .rsid("rs11540652")
        .with_snpeff("NM_000546.5", "c.743G>A", Some("p.Arg248Gln"))
        .with_snpeff("NM_001126118.1", "c.626G>A", Some("p.Arg209Gln"))
        .hit(
            "chr17:g.7577538C>T",
            "TP53",
            "p.Arg248Gln, p.R116Q, p.R248Q, p.R209Q",
        );
    let error = mane_numbering_refusal_message(
        "TP53 R209Q",
        "TP53",
        "R209Q",
        "P04637",
        "Arg",
        209,
        "p.Arg248Gln",
        "NM_000546.6",
        &hit,
        Some("NM_000546.6"),
    );
    let BioMcpError::InvalidArgument(message) = &error else {
        panic!("a MANE-numbered request refuses as invalid argument, got: {error}");
    };
    for expected in [
        "No MANE-numbered variant matches 'TP53 R209Q'",
        "canonical protein (UniProt P04637) has Arg at 209",
        "no matching record names that change",
        "the only alias match is p.Arg248Gln on NM_000546.6",
        "the request is spelled p.Arg209Gln on NM_001126118.1, another transcript",
        "Retry `biomcp get variant` with a transcript-qualified HGVS naming 'R209Q'",
        "biomcp search variant -g TP53 --hgvsp R209Q",
    ] {
        assert!(
            message.contains(expected),
            "missing {expected:?} in: {message}"
        );
    }
    assert!(
        !message.contains("Candidates:"),
        "the refusal carries no bare candidate list (ticket 2042): {message}"
    );
    assert!(
        !message.contains("follows another transcript's numbering"),
        "the refusal must not carry the other-transcript note: {message}"
    );
    assert!(
        !message.contains("one candidate's exact form"),
        "the retry line must not point at the known-differing candidate: {message}"
    );
}

/// Recorded BRCA1 Y1866D shape (query_brca1_y1866d_20261009.json): the
/// request's position 1866 lies past the 1863 residues of the canonical
/// protein P38398, so no residue comparison can run — but a position the
/// protein cannot hold proves by itself that the request follows another
/// numbering. The answer keeps the MANE headline (ClinVar 869017 names
/// NM_007294.4 p.Tyr1845Asp) and prints the other-transcript note instead
/// of resolving silently (ticket 2042).
#[test]
fn protein_change_resolution_notes_a_position_past_the_canonical_protein_end() {
    let hits = vec![
        ProteinHitBuilder::default()
            .clinvar(869017, "NM_007294.4(BRCA1):c.5533T>G (p.Tyr1845Asp)")
            .with_snpeff("NM_007300.3", "c.5596T>G", Some("p.Tyr1866Asp"))
            .with_snpeff("NM_007298.3", "c.2221T>G", Some("p.Tyr741Asp"))
            .with_snpeff("NM_007294.3", "c.5533T>G", Some("p.Tyr1845Asp"))
            .hit(
                "chr17:g.41197754A>C",
                "BRCA1",
                "p.Tyr1845Asp, p.Y1845D, p.Y1866D, p.Tyr1866Asp",
            ),
    ];
    let mane = transform::variant::clinvar_mane_transcript(&hits);
    assert_eq!(mane.as_deref(), Some("NM_007294.4"));

    let resolved =
        resolve_protein_change_hit("BRCA1 Y1866D", "BRCA1", "Y1866D", hits, mane.as_deref())
            .expect("a unique provider hit resolves before the numbering check");
    let facts = canonical_facts("P38398", None, 1863, "NM_007294.4");
    let note = refuse_or_note_with_facts(
        "BRCA1 Y1866D",
        "BRCA1",
        "Y1866D",
        &resolved,
        mane.as_deref(),
        &facts,
    )
    .expect("a position past the canonical end prints its note");
    assert_eq!(
        note,
        Some(
            "Numbering note: resolved on NM_007294.4 as p.Tyr1845Asp; the \
             requested Y1866D follows another transcript's numbering."
                .to_string(),
        )
    );
}

/// Recorded BRCA1 Q1878R shape (query_brca1_q1878r_20261009.json): the
/// ClinVar-less hit's SnpEff list carries 48 protein-structure
/// interaction rows beside the transcript rows, and the request's
/// position 1878 again lies past the canonical protein's 1863 residues.
/// The MANE-Select cross-reference (NM_007294.4) headlines
/// p.Gln1857Arg and the past-the-end note prints (ticket 2042); the
/// interaction rows must not cost the record its whole annotation list.
#[test]
fn protein_change_resolution_notes_a_many_row_record_past_the_protein_end() {
    let mut builder = ProteinHitBuilder::default();
    for index in 0..48 {
        builder = builder.with_interaction(
            &format!("1JNX:X_182{index}-X_1857:NM_007294.3"),
            "c.5570A>G",
        );
    }
    let hits = vec![
        builder
            .with_snpeff("NM_007300.3", "c.5633A>G", Some("p.Gln1878Arg"))
            .with_snpeff("NM_007298.3", "c.2258A>G", Some("p.Gln753Arg"))
            .with_snpeff("NM_007297.3", "c.5429A>G", Some("p.Gln1810Arg"))
            .with_snpeff("NM_007294.3", "c.5570A>G", Some("p.Gln1857Arg"))
            .hit(
                "chr17:g.41197717T>C",
                "BRCA1",
                "p.Gln1878Arg, p.Q1878R, p.Q1857R, p.Gln1857Arg",
            ),
    ];
    assert_eq!(transform::variant::clinvar_mane_transcript(&hits), None);

    let resolved = resolve_protein_change_hit("BRCA1 Q1878R", "BRCA1", "Q1878R", hits, None)
        .expect("a unique provider hit resolves before the numbering check");
    let facts = canonical_facts("P38398", None, 1863, "NM_007294.4");
    let note = refuse_or_note_with_facts(
        "BRCA1 Q1878R",
        "BRCA1",
        "Q1878R",
        &resolved,
        facts.mane_transcript.as_deref(),
        &facts,
    )
    .expect("a position past the canonical end prints its note");
    assert_eq!(
        note,
        Some(
            "Numbering note: resolved on NM_007294.4 as p.Gln1857Arg; the \
             requested Q1878R follows another transcript's numbering."
                .to_string(),
        )
    );
    let variant = transform::variant::from_myvariant_hit_with_mane(
        &resolved,
        facts.mane_transcript.as_deref(),
        Some("Q1878R"),
    );
    assert_eq!(variant.hgvs_p.as_deref(), Some("p.Gln1857Arg"));
    assert_eq!(variant.transcript.as_deref(), Some("NM_007294.4"));
}

/// Recorded TP53 S183Y shape with UniProt unreachable (ticket 2042): the
/// canonical facts never arrive, so the residue check cannot run. The
/// headline still spells the request (the shorter isoform's
/// p.Ser183Tyr), so the answer resolves under a note that says plainly
/// the numbering could not be checked against MANE — never silently.
#[test]
fn protein_change_resolution_notes_unchecked_numbering_without_facts() {
    let hits = vec![
        ProteinHitBuilder::default()
            .rsid("rs2073157445")
            .with_snpeff("NM_000546.5", "c.944C>A", Some("p.Ser315Tyr"))
            .with_snpeff("NM_001126115.1", "c.548C>A", Some("p.Ser183Tyr"))
            .with_snpeff("NM_001276697.1", "c.467C>A", Some("p.Ser156Tyr"))
            .hit(
                "chr17:g.7576902G>T",
                "TP53",
                "p.S183Y, p.S156Y, p.S315Y, p.Ser315Tyr, p.Ser183Tyr",
            ),
    ];
    let resolved = resolve_protein_change_hit("TP53 S183Y", "TP53", "S183Y", hits, None)
        .expect("a unique provider hit resolves before the numbering check");
    let note = refuse_or_note_without_facts("TP53 S183Y", "TP53", "S183Y", &resolved, None)
        .expect("an unchecked spelling of the request resolves with its note");
    assert_eq!(
        note,
        Some(
            "Numbering note: resolved on NM_001126115.1 as p.Ser183Tyr; the \
             requested S183Y could not be checked against MANE numbering."
                .to_string(),
        )
    );
}

/// Recorded TP53 R116Q shape with the canonical facts unavailable (ticket
/// 2042): the headline names a different change (p.Arg248Gln), and
/// without the residue check BioMCP cannot tell a MANE-numbered request
/// that no record names from another transcript's numbering — so it
/// refuses rather than return the lookalike with no note at all.
#[test]
fn protein_change_resolution_refuses_a_lookalike_without_facts() {
    let hits = vec![
        ProteinHitBuilder::default()
            .clinvar(12356, "NM_000546.6(TP53):c.743G>A (p.Arg248Gln)")
            .with_snpeff("NM_000546.5", "c.743G>A", Some("p.Arg248Gln"))
            .with_snpeff("NM_001126115.1", "c.347G>A", Some("p.Arg116Gln"))
            .hit(
                "chr17:g.7577538C>T",
                "TP53",
                "p.Arg248Gln, p.R116Q, p.R248Q",
            ),
    ];
    let mane = transform::variant::clinvar_mane_transcript(&hits);
    assert_eq!(mane.as_deref(), Some("NM_000546.6"));
    let resolved = resolve_protein_change_hit("TP53 R116Q", "TP53", "R116Q", hits, mane.as_deref())
        .expect("a unique provider hit resolves before the numbering check");
    let error =
        refuse_or_note_without_facts("TP53 R116Q", "TP53", "R116Q", &resolved, mane.as_deref())
            .expect_err("a differing headline refuses when the numbering cannot be checked");
    let BioMcpError::InvalidArgument(message) = &error else {
        panic!("the unchecked lookalike refuses as invalid argument, got: {error}");
    };
    for expected in [
        "No MANE-numbered variant matches 'TP53 R116Q'",
        "the canonical (MANE Select) protein could not be read",
        "the only alias match is p.Arg248Gln on NM_000546.6",
        "the request is spelled p.Arg116Gln on NM_001126115.1, another transcript",
        "Retry `biomcp get variant` with a transcript-qualified HGVS naming 'R116Q'",
    ] {
        assert!(
            message.contains(expected),
            "missing {expected:?} in: {message}"
        );
    }
    assert!(
        !message.contains("Candidates:"),
        "the refusal carries no bare candidate list (ticket 2042): {message}"
    );
    assert!(
        !message.contains("follows another transcript's numbering"),
        "the refusal must not claim a numbering it could not check: {message}"
    );
}

/// The same S183Y shape with the canonical facts in hand but no MANE
/// transcript named by the record: the residue check passes (P04637 has
/// Ser at 183) while nothing marks the headline transcript as MANE, so
/// the answer keeps the spelled request and says plainly the numbering
/// could not be checked against MANE (ticket 2042).
#[test]
fn protein_change_resolution_notes_unchecked_numbering_without_a_mane_transcript() {
    let hits = vec![
        ProteinHitBuilder::default()
            .rsid("rs2073157445")
            .with_snpeff("NM_000546.5", "c.944C>A", Some("p.Ser315Tyr"))
            .with_snpeff("NM_001126115.1", "c.548C>A", Some("p.Ser183Tyr"))
            .with_snpeff("NM_001276697.1", "c.467C>A", Some("p.Ser156Tyr"))
            .hit(
                "chr17:g.7576902G>T",
                "TP53",
                "p.S183Y, p.S156Y, p.S315Y, p.Ser315Tyr, p.Ser183Tyr",
            ),
    ];
    let resolved = resolve_protein_change_hit("TP53 S183Y", "TP53", "S183Y", hits, None)
        .expect("a unique provider hit resolves before the numbering check");
    let facts = CanonicalProteinFacts {
        accession: "P04637".into(),
        residue: Some('S'),
        sequence_length: Some(393),
        mane_transcript: None,
    };
    let note = refuse_or_note_with_facts("TP53 S183Y", "TP53", "S183Y", &resolved, None, &facts)
        .expect("an unmarked MANE transcript prints the unchecked note");
    assert_eq!(
        note,
        Some(
            "Numbering note: resolved on NM_001126115.1 as p.Ser183Tyr; the \
             requested S183Y could not be checked against MANE numbering."
                .to_string(),
        )
    );
}

/// A protein-change request whose resolved record names no protein change
/// on any headline transcript refuses instead of answering a bare genomic
/// variant (ticket 2042): nothing can carry the numbering story.
#[test]
fn protein_change_resolution_refuses_a_hit_without_any_protein_change() {
    let hits = vec![ProteinHitBuilder::default().rsid("rs9999001").hit(
        "chr17:g.41197717T>C",
        "BRCA1",
        "p.Q1878R, p.Q1857R",
    )];
    let resolved = resolve_protein_change_hit("BRCA1 Q1878R", "BRCA1", "Q1878R", hits, None)
        .expect("a unique provider hit resolves before the numbering check");
    let facts = canonical_facts("P38398", None, 1863, "NM_007294.4");
    let error = refuse_or_note_with_facts(
        "BRCA1 Q1878R",
        "BRCA1",
        "Q1878R",
        &resolved,
        facts.mane_transcript.as_deref(),
        &facts,
    )
    .expect_err("a record with no protein change refuses");
    let BioMcpError::InvalidArgument(message) = &error else {
        panic!("the protein-change-less hit refuses as invalid argument, got: {error}");
    };
    for expected in [
        "No protein change answers 'BRCA1 Q1878R'",
        "offers no protein change on a transcript BioMCP can headline",
        "Retry `biomcp get variant` with a transcript-qualified HGVS naming 'Q1878R'",
    ] {
        assert!(
            message.contains(expected),
            "missing {expected:?} in: {message}"
        );
    }
    assert!(!message.contains("Candidates:"), "{message}");
}

/// Ticket 2047: the same refusal with facts that name no MANE transcript
/// must not claim the record carries no protein change anywhere. The
/// record's rows carry protein changes on rows without coding spellings,
/// so none can be headlined; the wording says what BioMCP could not
/// headline instead of overclaiming, and the refusal still names the
/// isoform that spells the request.
#[test]
fn protein_change_absent_refusal_without_a_mane_transcript_claims_only_the_headline() {
    let hits = vec![
        ProteinHitBuilder::default()
            .rsid("rs9999002")
            .with_snpeff("NM_007300.3", "", Some("p.Gln1878Arg"))
            .with_snpeff("NM_007294.3", "", Some("p.Gln1857Arg"))
            .hit("chr17:g.41197717T>C", "BRCA1", "p.Q1878R, p.Q1857R"),
    ];
    let resolved = resolve_protein_change_hit("BRCA1 Q1878R", "BRCA1", "Q1878R", hits, None)
        .expect("a unique provider hit resolves before the numbering check");
    let facts = CanonicalProteinFacts {
        accession: "P38398".into(),
        residue: None,
        sequence_length: Some(1863),
        mane_transcript: None,
    };
    let error =
        refuse_or_note_with_facts("BRCA1 Q1878R", "BRCA1", "Q1878R", &resolved, None, &facts)
            .expect_err("a past-the-end request whose record headlines no protein change refuses");
    let BioMcpError::InvalidArgument(message) = &error else {
        panic!("the protein-change-less hit refuses as invalid argument, got: {error}");
    };
    assert!(
        message.contains("offers no protein change on a transcript BioMCP can headline"),
        "{message}"
    );
    assert!(
        message.contains("the request is spelled p.Gln1878Arg on NM_007300.3, another transcript"),
        "the refusal still names the isoform that spells the request: {message}"
    );
    assert!(
        !message.contains("no protein change on any transcript"),
        "the refusal must not claim the record carries no protein change anywhere: {message}"
    );
}

/// Ticket 2047's mid-sequence arm: the request's residue matches the
/// canonical protein, so its numbering is valid there, while the record
/// offers no protein change on a transcript BioMCP can headline. The
/// refusal must fire on this arm too — the only prior test reached the
/// refusal through the past-the-end path, so deleting the mid-sequence
/// refusal kept every test green.
#[test]
fn protein_change_absent_refusal_fires_mid_sequence() {
    let hits = vec![
        ProteinHitBuilder::default()
            .rsid("rs9999003")
            .with_snpeff("NM_007300.3", "", Some("p.Gln1878Arg"))
            .hit("chr17:g.41223234A>T", "BRCA1", "p.S1587F, p.Ser1587Phe"),
    ];
    let resolved = resolve_protein_change_hit("BRCA1 S1587F", "BRCA1", "S1587F", hits, None)
        .expect("a unique provider hit resolves before the numbering check");
    let facts = CanonicalProteinFacts {
        accession: "P38398".into(),
        residue: Some('S'),
        sequence_length: Some(1863),
        mane_transcript: None,
    };
    let error =
        refuse_or_note_with_facts("BRCA1 S1587F", "BRCA1", "S1587F", &resolved, None, &facts)
            .expect_err(
                "a valid-numbering request whose record headlines no protein change refuses",
            );
    let BioMcpError::InvalidArgument(message) = &error else {
        panic!("the mid-sequence arm refuses as invalid argument, got: {error}");
    };
    assert!(
        message.contains("No protein change answers 'BRCA1 S1587F'"),
        "{message}"
    );
    assert!(
        message.contains("offers no protein change on a transcript BioMCP can headline"),
        "{message}"
    );
}

/// Ticket 2047: with the canonical facts in hand but no MANE
/// cross-reference, a request whose reference residue differs from the
/// canonical protein still resolved its isoform spelling silently — the
/// residue check had proven another numbering, but the equivalent
/// headline skipped the note. The answer now names the transcript it
/// resolved on and says the request follows another transcript's
/// numbering.
#[test]
fn protein_change_resolution_notes_a_proven_other_numbering_without_a_mane_transcript() {
    // Recorded TP53 S183Y shape (query_tp53_s183y_20261009.json) with
    // facts whose residue at 183 differs from the request's Ser: the
    // residue check proves another numbering while the record's headline
    // still spells the request on the shorter isoform.
    let hits = vec![
        ProteinHitBuilder::default()
            .rsid("rs2073157445")
            .with_snpeff("NM_000546.5", "c.944C>A", Some("p.Ser315Tyr"))
            .with_snpeff("NM_001126115.1", "c.548C>A", Some("p.Ser183Tyr"))
            .with_snpeff("NM_001276697.1", "c.467C>A", Some("p.Ser156Tyr"))
            .hit(
                "chr17:g.7576902G>T",
                "TP53",
                "p.S183Y, p.S156Y, p.S315Y, p.Ser315Tyr, p.Ser183Tyr",
            ),
    ];
    let resolved = resolve_protein_change_hit("TP53 S183Y", "TP53", "S183Y", hits, None)
        .expect("a unique provider hit resolves before the numbering check");
    let facts = CanonicalProteinFacts {
        accession: "P04637".into(),
        // The canonical protein holds Pro at 183, not Ser: the residue
        // check proves the request follows another transcript's numbering.
        residue: Some('P'),
        sequence_length: Some(393),
        mane_transcript: None,
    };
    let note = refuse_or_note_with_facts("TP53 S183Y", "TP53", "S183Y", &resolved, None, &facts)
        .expect("a proven other numbering prints its note, never a silent isoform answer");
    assert_eq!(
        note,
        Some(
            "Numbering note: resolved on NM_001126115.1 as p.Ser183Tyr; the \
             requested S183Y follows another transcript's numbering."
                .to_string(),
        )
    );
}

/// Recorded BRCA1 H1883D shape (ticket 2042's named second past-the-end
/// probe): the ClinVar-less hit spells the request on the long isoform
/// (NM_007300.3 p.His1883Asp) while MANE Select NM_007294 names
/// p.His1862Asp, and the request's position 1883 lies past the 1863
/// residues of P38398. The MANE-Select cross-reference headlines the MANE
/// row and the past-the-end note prints.
#[test]
fn protein_change_resolution_notes_h1883d_past_the_canonical_protein_end() {
    let hits = vec![
        ProteinHitBuilder::default()
            .rsid("rs761585448")
            .with_snpeff("NM_007300.3", "c.5647C>G", Some("p.His1883Asp"))
            .with_snpeff("NM_007298.3", "c.2272C>G", Some("p.His758Asp"))
            .with_snpeff("NM_007297.3", "c.5443C>G", Some("p.His1815Asp"))
            .with_snpeff("NM_007294.3", "c.5584C>G", Some("p.His1862Asp"))
            .with_snpeff("NM_007299.3", "c.*98C>G", None)
            .with_snpeff("NR_027676.1", "n.5720C>G", None)
            .hit(
                "chr17:g.41197703G>C",
                "BRCA1",
                "p.H1883D, p.His1883Asp, p.H1862D, p.His1862Asp, p.H758D",
            ),
    ];
    assert_eq!(transform::variant::clinvar_mane_transcript(&hits), None);

    let resolved = resolve_protein_change_hit("BRCA1 H1883D", "BRCA1", "H1883D", hits, None)
        .expect("a unique provider hit resolves before the numbering check");
    let facts = canonical_facts("P38398", None, 1863, "NM_007294.4");
    let note = refuse_or_note_with_facts(
        "BRCA1 H1883D",
        "BRCA1",
        "H1883D",
        &resolved,
        facts.mane_transcript.as_deref(),
        &facts,
    )
    .expect("a position past the canonical end prints its note");
    assert_eq!(
        note,
        Some(
            "Numbering note: resolved on NM_007294.4 as p.His1862Asp; the \
             requested H1883D follows another transcript's numbering."
                .to_string(),
        )
    );
    let variant = transform::variant::from_myvariant_hit_with_mane(
        &resolved,
        facts.mane_transcript.as_deref(),
        Some("H1883D"),
    );
    assert_eq!(variant.hgvs_p.as_deref(), Some("p.His1862Asp"));
    assert_eq!(variant.transcript.as_deref(), Some("NM_007294.4"));
}
