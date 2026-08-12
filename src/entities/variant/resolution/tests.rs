//! Sidecar tests for variant resolution helpers.

use super::*;
use crate::entities::variant::{GenomeBuild, resolved_default_assembly};

#[test]
fn versioned_refseq_rows_normalize_to_their_chromosome_and_build() {
    for (accession, chromosome, genome_build) in REFSEQ_GENOMIC_BUILDS {
        let coordinate = normalize_genomic_coordinate(&format!("{accession}:g.1A>T"))
            .expect("valid RefSeq coordinate")
            .expect("recognized RefSeq coordinate");
        assert_eq!(coordinate.id, format!("{chromosome}:g.1A>T"));
        assert_eq!(coordinate.genome_build, Some(*genome_build));
        assert!(!coordinate.requires_comparison);
    }
}

#[test]
fn coordinate_normalizer_handles_aliases_and_alternate_spellings() {
    let rows = [
        (
            "GRCh37:chr7:g.140453136A>T",
            "chr7:g.140453136A>T",
            Some(GenomeBuild::Grch37),
        ),
        (
            "hg38:CHR7:g.140753336a>t",
            "chr7:g.140753336A>T",
            Some(GenomeBuild::Grch38),
        ),
        ("chr10:87925512:G:A", "chr10:g.87925512G>A", None),
        (
            "NC_000010.11:87925511:G:A",
            "chr10:g.87925512G>A",
            Some(GenomeBuild::Grch38),
        ),
        (
            "NC_000010.11:g.87925512del",
            "chr10:g.87925512del",
            Some(GenomeBuild::Grch38),
        ),
    ];
    for (input, id, build) in rows {
        let coordinate = normalize_genomic_coordinate(input).unwrap().unwrap();
        assert_eq!(coordinate.id, id);
        assert_eq!(coordinate.genome_build, build);
    }
    assert!(normalize_genomic_coordinate("NC_000010:g.1A>T").is_err());
    assert!(normalize_genomic_coordinate("NC_000010.99:g.1A>T").is_err());
    assert!(normalize_genomic_coordinate("chr10:g.0A>T").is_err());
}

#[test]
#[serial_test::serial]
fn default_assembly_precedence_is_explicit_then_environment_then_grch38() {
    unsafe { std::env::remove_var("BIOMCP_DEFAULT_ASSEMBLY") };
    assert_eq!(
        resolved_default_assembly(None).unwrap(),
        GenomeBuild::Grch38
    );
    unsafe { std::env::set_var("BIOMCP_DEFAULT_ASSEMBLY", "hg19") };
    assert_eq!(
        resolved_default_assembly(None).unwrap(),
        GenomeBuild::Grch37
    );
    assert_eq!(
        resolved_default_assembly(Some(GenomeBuild::Grch38)).unwrap(),
        GenomeBuild::Grch38
    );
    unsafe { std::env::set_var("BIOMCP_DEFAULT_ASSEMBLY", "not-a-build") };
    assert!(resolved_default_assembly(None).is_err());
    assert_eq!(
        resolved_default_assembly(Some(GenomeBuild::Grch37)).unwrap(),
        GenomeBuild::Grch37
    );
    unsafe { std::env::remove_var("BIOMCP_DEFAULT_ASSEMBLY") };
}

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

#[test]
fn structured_article_identity_accepts_refseq_forms_and_preserves_chr_compatibility() {
    let components: VariantArticleRequest = serde_json::from_value(serde_json::json!({
        "gene": " ATM ",
        "transcript": " NM_000051.4 ",
        "coding": " c.1066-6T>G ",
        "accession": " NC_000011.10 ",
        "build": " GRCh38 ",
        "position": 108248927,
        "ref": " T ",
        "alt": " G "
    }))
    .expect("component request");
    let genomic: VariantArticleRequest = serde_json::from_value(serde_json::json!({
        "gene": "ATM",
        "transcript": "NM_000051.4",
        "coding": "c.1066-6T>G",
        "genomic": " NC_000011.10:g.108248927T>G ",
        "build": "GRCh38"
    }))
    .expect("genomic request");
    let component_identity = components.validate_identity().expect("valid components");
    assert_eq!(
        component_identity,
        genomic.validate_identity().expect("valid HGVS")
    );
    assert!(component_identity.is_authoritative_refseq());
    assert_eq!(component_identity.gene.as_deref(), Some("ATM"));

    let chr_request: VariantArticleRequest = serde_json::from_value(serde_json::json!({
        "genomic": "chr7:g.140453136A>T",
        "build": "GRCh38"
    }))
    .expect("chr request");
    assert!(chr_request.validate_identity().is_ok());
}

#[test]
fn structured_article_identity_requires_refseq_build_and_rejects_duplicate_components() {
    let without_build: VariantArticleRequest = serde_json::from_value(serde_json::json!({
        "genomic": "NC_000011.10:g.108248927T>G"
    }))
    .expect("request");
    assert!(
        without_build
            .validate_identity()
            .unwrap_err()
            .to_string()
            .contains("explicit")
    );

    let duplicate: VariantArticleRequest = serde_json::from_value(serde_json::json!({
        "genomic": "NC_000011.10:g.108248927T>G",
        "build": "GRCh38",
        "position": 108248927
    }))
    .expect("request");
    assert!(
        duplicate
            .validate_identity()
            .unwrap_err()
            .to_string()
            .contains("cannot combine")
    );
}

fn source_identity() -> SourceVariantIdentity {
    SourceVariantIdentity {
        genomic_id: "GRCh38:chr7:g.140453136A>T".into(),
        genome_build: "GRCh38".into(),
        genes: vec!["BRAF".into()],
        protein_changes: vec!["NP_004324.2:p.Val600Glu".into(), "p.V600E".into()],
        coding_changes: vec!["NM_004333.6:c.1799T>A".into()],
        rsids: vec!["rs113488022".into()],
    }
}

#[test]
fn protein_normalization_preserves_identity_across_supported_spellings() {
    for alias in ["V600E", "p.V600E", "p.Val600Glu", "NP_004324.2:p.Val600Glu"] {
        assert_eq!(normalize_protein_change(alias).as_deref(), Some("V600E"));
    }
    for alias in ["L39*", "p.Leu39Ter", "NP_000001.1:p.Leu39Stop"] {
        assert_eq!(normalize_protein_change(alias).as_deref(), Some("L39*"));
    }
    assert_ne!(
        normalize_protein_change("V601E"),
        normalize_protein_change("V600E")
    );
    assert_ne!(
        normalize_protein_change("V600K"),
        normalize_protein_change("V600E")
    );
}

#[test]
fn identity_comparison_accepts_identical_complex_protein_hgvs() {
    let requested = RequestedVariantIdentity::for_search(
        Some("EGFR".into()),
        Some("p.Glu746_Ala750del".into()),
        None,
        None,
    );
    let source = SourceVariantIdentity {
        genomic_id: "chr7:g.55242465_55242479del".into(),
        genome_build: "GRCh37".into(),
        genes: vec!["EGFR".into()],
        protein_changes: vec!["NP_005219.2:p.Glu746_Ala750del".into()],
        ..Default::default()
    };

    assert_eq!(
        compare_variant_identity(&requested, &source),
        VariantIdentityComparison::Compatible {
            matched_alias: "NP_005219.2:p.Glu746_Ala750del".into(),
        }
    );

    let contradictory = RequestedVariantIdentity {
        protein_change: Some("p.Glu746_Ala751del".into()),
        ..requested
    };
    assert_eq!(
        compare_variant_identity(&contradictory, &source),
        VariantIdentityComparison::Contradictory {
            field: "protein_change"
        }
    );
}

#[test]
fn identity_comparison_preserves_provider_alias_and_checks_every_known_field() {
    let requested = RequestedVariantIdentity {
        gene: Some("BRAF".into()),
        protein_change: Some("p.Val600Glu".into()),
        coding_change: Some("c.1799T>A".into()),
        transcript: Some("NM_004333.6".into()),
        genomic_accession: Some("chr7".into()),
        genome_build: Some("GRCh38".into()),
        position: Some(140453136),
        reference: Some("A".into()),
        alternate: Some("T".into()),
        rsid: Some("RS113488022".into()),
    };
    assert_eq!(
        compare_variant_identity(&requested, &source_identity()),
        VariantIdentityComparison::Compatible {
            matched_alias: "NP_004324.2:p.Val600Glu".into()
        }
    );

    let cases = [
        (
            "gene",
            RequestedVariantIdentity {
                gene: Some("EGFR".into()),
                ..requested.clone()
            },
        ),
        (
            "protein_change",
            RequestedVariantIdentity {
                protein_change: Some("V601E".into()),
                ..requested.clone()
            },
        ),
        (
            "coding_change",
            RequestedVariantIdentity {
                coding_change: Some("c.1799T>G".into()),
                ..requested.clone()
            },
        ),
        (
            "transcript",
            RequestedVariantIdentity {
                transcript: Some("NM_999999.1".into()),
                ..requested.clone()
            },
        ),
        (
            "genome_build",
            RequestedVariantIdentity {
                genome_build: Some("GRCh37".into()),
                ..requested.clone()
            },
        ),
        (
            "genomic_accession",
            RequestedVariantIdentity {
                genomic_accession: Some("chr8".into()),
                ..requested.clone()
            },
        ),
        (
            "position",
            RequestedVariantIdentity {
                position: Some(140453137),
                ..requested.clone()
            },
        ),
        (
            "reference",
            RequestedVariantIdentity {
                reference: Some("G".into()),
                ..requested.clone()
            },
        ),
        (
            "alternate",
            RequestedVariantIdentity {
                alternate: Some("C".into()),
                ..requested.clone()
            },
        ),
        (
            "rsid",
            RequestedVariantIdentity {
                rsid: Some("rs1".into()),
                ..requested
            },
        ),
    ];
    for (field, request) in cases {
        assert_eq!(
            compare_variant_identity(&request, &source_identity()),
            VariantIdentityComparison::Contradictory { field },
            "field={field}"
        );
    }
}

#[test]
fn identity_comparison_is_indeterminate_for_missing_or_unlinked_annotation_evidence() {
    let request = RequestedVariantIdentity {
        gene: Some("BRAF".into()),
        protein_change: Some("V600E".into()),
        ..Default::default()
    };
    let mut source = source_identity();
    source.protein_changes.clear();
    assert_eq!(
        compare_variant_identity(&request, &source),
        VariantIdentityComparison::Indeterminate {
            field: "protein_change"
        }
    );

    let mut source = source_identity();
    source.genes.push("ARAF".into());
    assert_eq!(
        compare_variant_identity(&request, &source),
        VariantIdentityComparison::Indeterminate {
            field: "gene_annotation_tuple"
        }
    );
}
