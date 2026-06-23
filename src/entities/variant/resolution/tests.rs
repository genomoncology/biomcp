//! Sidecar tests for variant resolution helpers.

use super::*;

#[test]
fn parse_variant_id_examples() {
    match parse_variant_id("rs113488022").unwrap() {
        VariantIdFormat::RsId(v) => assert_eq!(v, "rs113488022"),
        _ => panic!("expected rsid"),
    }
    match parse_variant_id("chr7:g.140453136A>T").unwrap() {
        VariantIdFormat::HgvsGenomic(v) => assert_eq!(v, "chr7:g.140453136A>T"),
        _ => panic!("expected hgvs"),
    }
    match parse_variant_id("BRAF V600E").unwrap() {
        VariantIdFormat::GeneProteinChange { gene, change } => {
            assert_eq!(gene, "BRAF");
            assert_eq!(change, "V600E");
        }
        _ => panic!("expected gene+protein"),
    }
}

#[test]
fn parse_variant_id_egfr_l858r() {
    match parse_variant_id("EGFR L858R").unwrap() {
        VariantIdFormat::GeneProteinChange { gene, change } => {
            assert_eq!(gene, "EGFR");
            assert_eq!(change, "L858R");
        }
        _ => panic!("expected gene+protein"),
    }
}

#[test]
fn parse_variant_id_kras_g12c() {
    match parse_variant_id("KRAS G12C").unwrap() {
        VariantIdFormat::GeneProteinChange { gene, change } => {
            assert_eq!(gene, "KRAS");
            assert_eq!(change, "G12C");
        }
        _ => panic!("expected gene+protein"),
    }
}

#[test]
fn parse_variant_id_normalizes_uppercase_rsid_prefix() {
    match parse_variant_id("RS113488022").unwrap() {
        VariantIdFormat::RsId(v) => assert_eq!(v, "rs113488022"),
        _ => panic!("expected rsid"),
    }
}

#[test]
fn parse_variant_id_accepts_long_form_gene_protein_change() {
    match parse_variant_id("BRAF p.Val600Glu").unwrap() {
        VariantIdFormat::GeneProteinChange { gene, change } => {
            assert_eq!(gene, "BRAF");
            assert_eq!(change, "V600E");
        }
        _ => panic!("expected gene+protein"),
    }
}

#[test]
fn parse_variant_id_accepts_prefixed_short_gene_protein_change() {
    match parse_variant_id("BRAF p.V600E").unwrap() {
        VariantIdFormat::GeneProteinChange { gene, change } => {
            assert_eq!(gene, "BRAF");
            assert_eq!(change, "V600E");
        }
        _ => panic!("expected gene+protein"),
    }
}

#[test]
fn classify_variant_input_detects_search_only_shorthand() {
    match classify_variant_input("PTPN22 620W") {
        VariantInputKind::Shorthand(VariantShorthand::GeneResidueAlias {
            gene,
            alias,
            position,
            residue,
        }) => {
            assert_eq!(gene, "PTPN22");
            assert_eq!(alias, "620W");
            assert_eq!(position, 620);
            assert_eq!(residue, 'W');
        }
        other => panic!("expected gene residue alias, got {other:?}"),
    }

    match classify_variant_input("R620W") {
        VariantInputKind::Shorthand(VariantShorthand::ProteinChangeOnly { change }) => {
            assert_eq!(change, "R620W");
        }
        other => panic!("expected protein change shorthand, got {other:?}"),
    }
}

#[test]
fn classify_variant_input_detects_transcript_coding_hgvs_before_rejecting() {
    match classify_variant_input("NM_004333.6:c.1799T>A") {
        VariantInputKind::TranscriptCodingHgvs(value) => {
            assert_eq!(value, "NM_004333.6:c.1799T>A");
        }
        other => panic!("expected transcript coding HGVS, got {other:?}"),
    }
}

#[test]
fn parse_variant_id_points_transcript_hgvs_to_normalize_when_direct_parse_is_used() {
    let message = parse_variant_id("NM_004333.6:c.1799T>A")
        .unwrap_err()
        .to_string();
    assert!(message.contains("transcript HGVS"));
    assert!(message.contains("biomcp variant normalize all NM_004333.6:c.1799T>A"));
    assert!(message.contains("Transcript HGVS: NM_004333.6:c.1799T>A"));
}

#[test]
fn classify_variant_input_normalizes_long_form_single_token_protein_change() {
    match classify_variant_input("p.Val600Glu") {
        VariantInputKind::Shorthand(VariantShorthand::ProteinChangeOnly { change }) => {
            assert_eq!(change, "V600E");
        }
        other => panic!("expected protein change shorthand, got {other:?}"),
    }
}

#[test]
fn parse_variant_id_points_search_only_shorthand_to_search_variant() {
    let residue_alias = parse_variant_id("PTPN22 620W").unwrap_err().to_string();
    assert!(residue_alias.contains("search-only shorthand"));
    assert!(residue_alias.contains("biomcp search variant \"PTPN22 620W\""));

    let protein_change_only = parse_variant_id("R620W").unwrap_err().to_string();
    assert!(protein_change_only.contains("search-only shorthand"));
    assert!(protein_change_only.contains("biomcp search variant --hgvsp R620W"));
}

#[test]
fn parse_variant_id_points_long_form_single_token_to_search_variant() {
    let protein_change_only = parse_variant_id("p.Val600Glu").unwrap_err().to_string();
    assert!(protein_change_only.contains("search-only shorthand"));
    assert!(protein_change_only.contains("biomcp search variant --hgvsp V600E"));
}

#[test]
fn parse_variant_id_suggests_search_for_complex_alteration_text() {
    let message = match parse_variant_id("EGFR Exon 19 Deletion") {
        Ok(_) => panic!("expected complex alteration text to be rejected"),
        Err(err) => err.to_string(),
    };
    assert!(message.contains("search phrase or alteration description"));
    assert!(message.contains("biomcp search variant \"EGFR Exon 19 Deletion\""));
}
