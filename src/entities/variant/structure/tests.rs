//! Sidecar tests for the variant structure helper: the residue check
//! against the chosen protein, and the section-outcome matrices.

use super::*;
use crate::entities::section_outcome::SectionOutcomeState;
use std::sync::{Arc, Mutex};

#[test]
fn hgvsp_position_extracts_three_letter_short_and_accession_prefixed_aliases() {
    assert_eq!(hgvsp_position("p.Val600Glu"), Some(600));
    assert_eq!(hgvsp_position("p.V600E"), Some(600));
    assert_eq!(hgvsp_position("NP_004324.2:p.Val600Glu"), Some(600));
}

#[test]
fn normalize_hgvsp_change_matches_accession_prefixed_aliases() {
    assert_eq!(
        crate::entities::variant::normalize_protein_change("NP_004324.2:p.Val600Glu").as_deref(),
        Some("V600E")
    );
}

#[test]
fn domain_result_matrix_classifies_contact_and_applicability() {
    let domain = VariantStructureDomain {
        accession: "IPR000719".into(),
        name: Some("Protein kinase domain".into()),
        domain_type: Some("domain".into()),
        start: 457,
        end: 717,
        source: "InterPro".into(),
    };
    let cases = [
        (Ok(None), SectionOutcomeState::Inapplicable, 0, false),
        (Ok(Some(Vec::new())), SectionOutcomeState::Empty, 0, true),
        (Ok(Some(vec![domain])), SectionOutcomeState::Data, 1, true),
        (
            Err(BioMcpError::Api {
                api: "InterPro".into(),
                message: "fixture failure".into(),
            }),
            SectionOutcomeState::Unavailable,
            0,
            false,
        ),
    ];

    for (result, expected_state, expected_rows, credited) in cases {
        let (domains, outcome) = apply_domains_result(result);
        assert_eq!(outcome.outcome(), expected_state);
        assert_eq!(domains.len(), expected_rows);
        assert_eq!(!outcome.sources().is_empty(), credited);
    }
}

#[test]
fn cancerhotspots_result_matrix_classifies_contact_and_applicability() {
    let recurrence = |position_count| CancerHotspotRecurrence {
        source: "cancerhotspots.org".into(),
        position_count,
        same_aa_count: None,
        matched_transcript: None,
    };
    let cases = [
        (Ok(None), SectionOutcomeState::Inapplicable, false, false),
        (
            Ok(Some(recurrence(None))),
            SectionOutcomeState::Empty,
            true,
            true,
        ),
        (
            Ok(Some(recurrence(Some(12)))),
            SectionOutcomeState::Data,
            true,
            true,
        ),
        (
            Err(BioMcpError::Api {
                api: "cancerhotspots.org".into(),
                message: "fixture failure".into(),
            }),
            SectionOutcomeState::Unavailable,
            false,
            false,
        ),
    ];

    for (result, expected_state, recurrence_present, credited) in cases {
        let (result_recurrence, outcome) = apply_cancerhotspots_result(result);
        assert_eq!(outcome.outcome(), expected_state);
        assert_eq!(result_recurrence.is_some(), recurrence_present);
        assert_eq!(!outcome.sources().is_empty(), credited);
    }
}

fn uniprot_record(sequence: Option<&str>) -> UniProtRecord {
    serde_json::from_value(serde_json::json!({
        "primaryAccession": "Q9UIF7",
        "sequence": sequence.map(|value| serde_json::json!({
            "length": value.chars().count() as u64,
            "value": value
        })),
    }))
    .expect("valid UniProt record")
}

fn structure_hit(gene: &str, aliases: &[&str]) -> crate::sources::myvariant::MyVariantHit {
    serde_json::from_value(serde_json::json!({
        "_id": "chr1:g.45797228C>T",
        "dbnsfp": {"genename": gene, "hgvsp": aliases}
    }))
    .expect("valid MyVariant hit")
}

fn gene_protein_change_request(gene: &str, change: &str) -> (Variant, VariantIdFormat) {
    (
        Variant {
            gene: gene.to_string(),
            hgvs_p: Some(change.to_string()),
            ..serde_json::from_value::<Variant>(serde_json::json!({
                "id": "chr1:g.45797228C>T",
                "gene": gene,
                "hgvs_p": change
            }))
            .expect("valid Variant")
        },
        VariantIdFormat::GeneProteinChange {
            gene: gene.to_string(),
            change: change.to_string(),
        },
    )
}

#[test]
fn reference_residue_check_verifies_mismatches_and_bounds() {
    // Q9UIF7 holds S at 368 and G at 393, like the recorded record.
    let mut sequence = "M".repeat(546).into_bytes();
    sequence[367] = b'S';
    sequence[392] = b'G';
    let sequence = String::from_utf8(sequence).expect("ascii sequence");
    let record = uniprot_record(Some(&sequence));

    assert_eq!(
        check_reference_residue(&record, 368, 'G'),
        ResidueCheck::Mismatch {
            position: 368,
            reference: 'G',
            actual: Some('S')
        }
    );
    assert_eq!(
        check_reference_residue(&record, 393, 'G'),
        ResidueCheck::Verified
    );
    // A position the sequence cannot hold proves the mismatch too.
    assert_eq!(
        check_reference_residue(&record, 600, 'G'),
        ResidueCheck::Mismatch {
            position: 600,
            reference: 'G',
            actual: None
        }
    );
    // A record without a sequence proves nothing either way.
    assert_eq!(
        check_reference_residue(&uniprot_record(None), 368, 'G'),
        ResidueCheck::Unchecked
    );
}

#[test]
fn mismatched_reference_residue_leaves_no_placement_and_reports_every_alias_position() {
    // The recorded MUTYH G368D aliases: the request matches at 368 while
    // Q9UIF7 holds S there, and 393 is the position that holds G — which a
    // letter match alone must not promote.
    let sequence = {
        let mut sequence = "M".repeat(546).into_bytes();
        sequence[367] = b'S';
        sequence[392] = b'G';
        String::from_utf8(sequence).expect("ascii sequence")
    };
    let record = uniprot_record(Some(&sequence));
    let (variant, id_format) = gene_protein_change_request("MUTYH", "G368D");
    let hit = structure_hit(
        "MUTYH",
        &["p.Gly393Asp", "p.G368D", "p.Gly368Asp", "p.G393D"],
    );

    let (residue, check) = residue_summary(&variant, &id_format, &hit, &record);

    assert_eq!(
        check,
        ResidueCheck::Mismatch {
            position: 368,
            reference: 'G',
            actual: Some('S')
        }
    );
    assert_eq!(residue.position, None, "no placement on the wrong protein");
    assert_eq!(residue.position_confidence, "unresolved");
    assert_eq!(residue.reference_aa.as_deref(), Some("G"));
    assert_eq!(
        residue.other_source_positions,
        vec![368, 393],
        "every alias position is an other-source position once none is selected"
    );
    assert_eq!(residue.matched_hgvsp.len(), 2);
}

#[test]
fn verified_reference_residue_keeps_the_exact_match_placement() {
    let sequence = {
        let mut sequence = "M".repeat(766).into_bytes();
        sequence[599] = b'V';
        String::from_utf8(sequence).expect("ascii sequence")
    };
    let record = uniprot_record(Some(&sequence));
    let (variant, id_format) = gene_protein_change_request("BRAF", "V600E");
    let hit = structure_hit("BRAF", &["p.Val640Glu", "p.Val600Glu", "p.V600E"]);

    let (residue, check) = residue_summary(&variant, &id_format, &hit, &record);

    assert_eq!(check, ResidueCheck::Verified);
    assert_eq!(residue.position, Some(600));
    assert_eq!(residue.position_confidence, "requested_hgvsp_exact_match");
    assert_eq!(residue.other_source_positions, vec![640]);
}

/// The recorded MyVariant.info query answer for `MUTYH G368D`: dbNSFP
/// spells the request at 368 on several isoform aliases while Q9UIF7's own
/// sequence holds S there and G at 393.
const MYVARIANT_MUTYH_G368D_QUERY_REPLY: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/testdata/sources/myvariant/query_mutyh_g368d_20261016.json"
));
const MYVARIANT_BRAF_V600E_QUERY_REPLY: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/testdata/sources/myvariant/query_braf_v600e_20261009.json"
));
/// The recorded UniProt records for the chosen proteins: Q9UIF7 (546
/// residues, S at 368) and P15056 (766 residues, V at 600), each minimized
/// to the accession, entry id, sequence, MANE-Select, one PDB row, and the
/// AlphaFoldDB cross-reference.
const UNIPROT_Q9UIF7_RECORD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/testdata/sources/uniprot/get_q9uif7_20261016.json"
));
const UNIPROT_P15056_RECORD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/testdata/sources/uniprot/get_p15056_20261016.json"
));
const CANCERHOTSPOTS_BRAF_REPLY: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/testdata/sources/cancerhotspots/by_gene_braf_20260805.json"
));
/// The gene-to-accession answer the resolver needs; the committed MyGene
/// captures carry the transcript fields this path never reads, so the test
/// spells the two fields the resolver does read.
const MYGENE_MUTYH_REPLY: &str =
    r#"{"total":1,"hits":[{"symbol":"MUTYH","uniprot":{"Swiss-Prot":"Q9UIF7"}}]}"#;
const MYGENE_BRAF_REPLY: &str =
    r#"{"total":1,"hits":[{"symbol":"BRAF","uniprot":{"Swiss-Prot":"P15056"}}]}"#;
/// The recorded InterPro answer shape, minimized to the one BRAF domain row
/// whose range carries residue 600.
const INTERPRO_P15056_REPLY: &str = r#"{"results":[{"metadata":{"accession":"IPR000719","name":"Protein kinase domain","type":"domain"},"proteins":[{"entry_protein_locations":[{"fragments":[{"start":457,"end":717}]}]}]}]}"#;
const CANCERHOTSPOTS_EMPTY_REPLY: &str = "[]";

type StructureReplyEnv = crate::test_support::FixtureEnv;

fn structure_recorded_reply(request: &str) -> (&'static str, &'static str, Vec<u8>) {
    let target = request.split_whitespace().nth(1).unwrap_or_default();
    if target.starts_with("/v1/query?") {
        if target.contains("dbnsfp.genename%3AMUTYH") && target.contains("p.G368D") {
            return (
                "200 OK",
                "application/json",
                MYVARIANT_MUTYH_G368D_QUERY_REPLY.to_vec(),
            );
        }
        if target.contains("dbnsfp.genename%3ABRAF") && target.contains("p.V600E") {
            return (
                "200 OK",
                "application/json",
                MYVARIANT_BRAF_V600E_QUERY_REPLY.to_vec(),
            );
        }
    }
    // The MyGene gene-to-accession resolver.
    if target.starts_with("/query?") {
        if target.contains("symbol%3A%22MUTYH%22") {
            return (
                "200 OK",
                "application/json",
                MYGENE_MUTYH_REPLY.as_bytes().to_vec(),
            );
        }
        if target.contains("symbol%3A%22BRAF%22") {
            return (
                "200 OK",
                "application/json",
                MYGENE_BRAF_REPLY.as_bytes().to_vec(),
            );
        }
    }
    if target.starts_with("/uniprotkb/Q9UIF7.json") {
        return ("200 OK", "application/json", UNIPROT_Q9UIF7_RECORD.to_vec());
    }
    if target.starts_with("/uniprotkb/P15056.json") {
        return ("200 OK", "application/json", UNIPROT_P15056_RECORD.to_vec());
    }
    if target.starts_with("/entry/interpro/protein/uniprot/P15056/") {
        return (
            "200 OK",
            "application/json",
            INTERPRO_P15056_REPLY.as_bytes().to_vec(),
        );
    }
    if target == "/api/hotspots/single/byGene/MUTYH" {
        return (
            "200 OK",
            "application/json",
            CANCERHOTSPOTS_EMPTY_REPLY.as_bytes().to_vec(),
        );
    }
    if target == "/api/hotspots/single/byGene/BRAF" {
        return (
            "200 OK",
            "application/json",
            CANCERHOTSPOTS_BRAF_REPLY.to_vec(),
        );
    }
    (
        "404 Not Found",
        "application/json",
        format!(r#"{{"error":"fixture path not found: {target}"}}"#).into_bytes(),
    )
}

/// One recorded-reply server for both structure rows: it replays the
/// committed captures above and logs every request line so tests can prove
/// which lookups ran.
async fn structure_recorded_reply_server() -> (
    String,
    Arc<Mutex<Vec<String>>>,
    crate::test_support::TestHttpServer,
) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    let task = crate::test_support::TestHttpServer::spawn(move |mut stream| {
        use tokio::io::{AsyncWriteExt};

            let captured = captured.clone();
        async move {
            // The production GET field list makes the request line span
            // TCP segments, so read until the headers end before routing.
            let Some(request) = crate::test_support::read_request(&mut stream).await? else {
                return Ok(());
            };
let request = String::from_utf8_lossy(&request).into_owned();
            captured.lock().expect("lock requests").push(
                request
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or_default()
                    .to_string(),
            );
            let (status, content_type, body) = structure_recorded_reply(&request);
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream
                .write_all(response.as_bytes())
                .await?;
            stream
                .write_all(&body)
                .await?;
            Ok(())
        }
    })
    .expect("start HTTP fixture");
    let base = task.base.clone();
    (base, requests, task)
}

fn structure_reply_env(base: &str, cache_dir: &tempfile::TempDir) -> StructureReplyEnv {
    let mut env = StructureReplyEnv::scratch_cache_only();
    env.set("BIOMCP_CACHE_MODE", "off");
    env.set_cache_root(cache_dir.path().to_str().expect("utf-8 temp path"));
    env.set("BIOMCP_MYVARIANT_BASE", format!("{base}/v1"));
    env.set("BIOMCP_MYGENE_BASE", base);
    env.set("BIOMCP_UNIPROT_BASE", base);
    env.set("BIOMCP_INTERPRO_BASE", base);
    env.set("BIOMCP_CANCERHOTSPOTS_BASE", base);
    env
}

/// The ticket's MUTYH row, replayed against the recorded providers: dbNSFP
/// spells `G368D` at 368 while Q9UIF7's own sequence holds S there, so the
/// structure helper must report the placement as unresolved, claim no
/// position on the protein, and suppress every residue-specific row while
/// the protein-level rows stay.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn mutyh_g368d_on_q9uif7_never_reports_position_368_as_a_placement() {
    let (base, requests, mut server) = structure_recorded_reply_server().await;
    let cache_dir = tempfile::tempdir().expect("temp cache dir");
    let _env = structure_reply_env(&base, &cache_dir);

    let result = crate::sources::with_no_cache(true, structure("MUTYH G368D"))
        .await
        .expect("recorded MUTYH G368D structure answer");
    server.finish().unwrap();
    let requests = requests.lock().expect("lock requests").join("\n");

    // The protein the helper chose still heads the answer.
    assert_eq!(result.protein.accession, "Q9UIF7");
    assert_eq!(result.protein.length, Some(546));
    assert_eq!(result.residue.reference_aa.as_deref(), Some("G"));
    assert_eq!(result.residue.requested_change.as_deref(), Some("G368D"));

    // No placement is claimed on the chosen protein: not the alias position,
    // not the exact-match label.
    assert_eq!(result.residue.position, None, "{:?}", result.residue);
    assert_eq!(result.residue.position_confidence, "unresolved");

    // Residue-specific rows are suppressed; protein-level rows stay.
    assert!(result.domains.is_empty(), "{:?}", result.domains);
    assert_eq!(
        result
            .lookup_outcomes
            .get("domains")
            .map(|outcome| outcome.outcome()),
        Some(SectionOutcomeState::Inapplicable)
    );
    assert!(
        !result.structures.pdb.is_empty(),
        "the protein-level PDB rows stay"
    );
    assert!(
        result
            .structures
            .pdb
            .iter()
            .all(|row| row.residue_covered.is_none()),
        "no PDB row claims coverage of an unplaced residue: {:?}",
        result.structures.pdb
    );
    assert!(
        result.structures.alphafold.is_some(),
        "the protein-level AlphaFold row stays"
    );

    assert!(
        result.cancerhotspots.is_none(),
        "unresolved placement has no hotspot counts"
    );
    assert_eq!(
        result
            .lookup_outcomes
            .get("cancerhotspots")
            .map(|outcome| outcome.outcome()),
        Some(SectionOutcomeState::Inapplicable)
    );

    // One warning names the finding plainly.
    assert_eq!(result.warnings.len(), 1, "{:?}", result.warnings);
    let warning = &result.warnings[0];
    assert!(warning.contains("Q9UIF7"), "{warning}");
    assert!(warning.contains("368"), "{warning}");
    assert!(warning.contains("S"), "{warning}");

    // The residue-specific lookups never ran: InterPro answers nothing
    // because no position selects a domain.
    assert!(
        !requests.contains("/entry/interpro/"),
        "InterPro must not be consulted without a placement: {requests}"
    );
    assert!(
        requests.contains("/uniprotkb/Q9UIF7.json"),
        "the chosen protein's record was read: {requests}"
    );
}

/// The ticket's BRAF row: the recorded P15056 sequence holds V at 600, so
/// the verified placement keeps its position, exact-match label, domain
/// row, PDB coverage claim, and hotspot join exactly as before.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn braf_v600e_on_p15056_keeps_the_verified_position_600() {
    let (base, requests, mut server) = structure_recorded_reply_server().await;
    let cache_dir = tempfile::tempdir().expect("temp cache dir");
    let _env = structure_reply_env(&base, &cache_dir);

    let result = crate::sources::with_no_cache(true, structure("BRAF V600E"))
        .await
        .expect("recorded BRAF V600E structure answer");
    server.finish().unwrap();
    let requests = requests.lock().expect("lock requests").join("\n");

    assert_eq!(result.protein.accession, "P15056");
    assert_eq!(result.protein.length, Some(766));
    assert_eq!(result.residue.position, Some(600));
    assert_eq!(
        result.residue.position_confidence,
        "requested_hgvsp_exact_match"
    );
    assert_eq!(result.residue.reference_aa.as_deref(), Some("V"));
    assert_eq!(result.residue.alternate_aa.as_deref(), Some("E"));
    assert_eq!(
        result.residue.other_source_positions,
        vec![207, 640],
        "the additional isoform positions stay reported: {:?}",
        result.residue
    );
    assert!(
        result
            .domains
            .iter()
            .any(|domain| domain.start <= 600 && domain.end >= 600),
        "{:?}",
        result.domains
    );
    assert!(
        result
            .structures
            .pdb
            .iter()
            .any(|row| row.residue_covered == Some(true)),
        "the verified position keeps its PDB coverage claim: {:?}",
        result.structures.pdb
    );
    assert!(result.structures.alphafold.is_some());
    assert_eq!(
        result
            .cancerhotspots
            .as_ref()
            .map(|row| row.source.as_str()),
        Some("cancerhotspots.org")
    );
    assert_eq!(
        result.warnings,
        vec![
            "MyVariant.info returned additional transcript/isoform protein positions; \
             mapped domain/structure context uses the requested HGVSp position."
                .to_string()
        ],
        "{:?}",
        result.warnings
    );
    assert!(
        requests.contains("/entry/interpro/protein/uniprot/P15056/"),
        "the verified placement still consults InterPro: {requests}"
    );
}
