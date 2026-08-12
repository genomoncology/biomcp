use clap::Parser;

mod articles;

use super::dispatch::{VariantSearchPlan, parse_simple_gene_change, resolve_variant_query};

use crate::cli::{
    Cli, Commands, GetEntity, OutputStream, SearchEntity, VariantCommand, run_outcome,
};
use crate::entities::variant as entity;

#[test]
fn search_variant_parses_single_token_positional_query() {
    let cli = Cli::try_parse_from(["biomcp", "search", "variant", "BRAF", "--limit", "2"])
        .expect("search variant should parse");

    let Cli {
        command:
            Commands::Search {
                entity:
                    SearchEntity::Variant(crate::cli::variant::VariantSearchArgs {
                        gene,
                        positional_query,
                        limit,
                        offset,
                        ..
                    }),
            },
        ..
    } = cli
    else {
        panic!("expected search variant command");
    };

    assert_eq!(gene, None);
    assert_eq!(positional_query, vec!["BRAF".to_string()]);
    assert_eq!(limit, 2);
    assert_eq!(offset, 0);
}

#[test]
fn search_variant_parses_multi_token_positional_query_and_flag() {
    let cli = Cli::try_parse_from([
        "biomcp", "search", "variant", "-g", "PTPN22", "R620W", "--limit", "5",
    ])
    .expect("search variant should parse");

    let Cli {
        command:
            Commands::Search {
                entity:
                    SearchEntity::Variant(crate::cli::variant::VariantSearchArgs {
                        gene,
                        positional_query,
                        limit,
                        ..
                    }),
            },
        ..
    } = cli
    else {
        panic!("expected search variant command");
    };

    assert_eq!(gene.as_deref(), Some("PTPN22"));
    assert_eq!(positional_query, vec!["R620W".to_string()]);
    assert_eq!(limit, 5);
}

#[test]
fn search_variant_parses_quoted_gene_change_positional_query() {
    let cli = Cli::try_parse_from(["biomcp", "search", "variant", "BRAF V600E", "--limit", "5"])
        .expect("search variant should parse");

    let Cli {
        command:
            Commands::Search {
                entity:
                    SearchEntity::Variant(crate::cli::variant::VariantSearchArgs {
                        positional_query,
                        limit,
                        ..
                    }),
            },
        ..
    } = cli
    else {
        panic!("expected search variant command");
    };

    assert_eq!(positional_query, vec!["BRAF V600E".to_string()]);
    assert_eq!(limit, 5);
}

#[test]
fn search_variant_help_distinguishes_exact_identity_from_broad_discovery() {
    let help = Cli::try_parse_from(["biomcp", "search", "variant", "--help"])
        .expect_err("help should stop parsing")
        .to_string();
    assert!(help.contains("reject contradictory source identities"));
    assert!(help.contains("structured resolution in JSON"));
    assert!(help.contains("Gene-only and discovery-filter searches remain broad"));
}

#[test]
fn variant_bare_id_parses_as_external_subcommand() {
    let cli = Cli::try_parse_from(["biomcp", "variant", "BRAF V600E"])
        .expect("bare variant id should parse");

    match cli.command {
        Commands::Variant {
            cmd: VariantCommand::External(args),
        } => assert_eq!(args, vec!["BRAF V600E"]),
        other => panic!("unexpected command: {other:?}"),
    }
}

#[test]
fn variant_trials_parses_source_flag() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "variant",
        "trials",
        "BRAF V600E",
        "--source",
        "nci",
        "--limit",
        "3",
    ])
    .expect("variant trials with --source should parse");

    match cli.command {
        Commands::Variant {
            cmd:
                VariantCommand::Trials {
                    source,
                    limit,
                    offset,
                    ..
                },
        } => {
            assert_eq!(source, "nci");
            assert_eq!(limit, 3);
            assert_eq!(offset, 0);
        }
        other => panic!("unexpected command: {other:?}"),
    }
}

#[tokio::test]
async fn handle_get_returns_guidance_json_for_shorthand_variant() {
    let cli = Cli::try_parse_from(["biomcp", "--json", "get", "variant", "R620W"]).expect("parse");

    let Cli {
        command: Commands::Get {
            entity: GetEntity::Variant(args),
        },
        json,
        ..
    } = cli
    else {
        panic!("expected get variant command");
    };

    let outcome = super::handle_get(args, json, false)
        .await
        .expect("guidance outcome");

    assert_eq!(outcome.stream, OutputStream::Stdout);
    assert_eq!(outcome.exit_code, 1);
    let value: serde_json::Value =
        serde_json::from_str(&outcome.text).expect("valid variant guidance json");
    assert_eq!(
        value["_meta"]["alias_resolution"]["kind"],
        "protein_change_only"
    );
    assert_eq!(
        value["_meta"]["next_commands"][0],
        "biomcp search variant --hgvsp R620W --limit 10"
    );
}

#[test]
fn parse_simple_gene_change_detects_supported_forms() {
    assert_eq!(
        parse_simple_gene_change("BRAF V600E"),
        Some(("BRAF".into(), "V600E".into()))
    );
    assert_eq!(
        parse_simple_gene_change("EGFR T790M"),
        Some(("EGFR".into(), "T790M".into()))
    );
    assert_eq!(
        parse_simple_gene_change("BRAF p.V600E"),
        Some(("BRAF".into(), "V600E".into()))
    );
    assert_eq!(
        parse_simple_gene_change("BRAF p.Val600Glu"),
        Some(("BRAF".into(), "V600E".into()))
    );
}

#[test]
fn parse_simple_gene_change_rejects_non_simple_forms() {
    assert_eq!(parse_simple_gene_change("BRAF"), None);
    assert_eq!(parse_simple_gene_change("EGFR Exon 19 Deletion"), None);
    assert_eq!(parse_simple_gene_change("EGFR Exon19"), None);
    assert_eq!(parse_simple_gene_change("braf V600E"), None);
}

#[test]
fn resolve_variant_query_maps_single_token_to_gene() {
    let resolved = resolve_variant_query(None, None, None, None, vec!["BRAF".into()]).unwrap();
    let VariantSearchPlan::Standard(resolved) = resolved else {
        panic!("expected standard search plan");
    };
    assert_eq!(resolved.gene.as_deref(), Some("BRAF"));
    assert!(resolved.hgvsp.is_none());
    assert!(resolved.hgvsc.is_none());
    assert!(resolved.rsid.is_none());
    assert!(resolved.condition.is_none());
    assert!(resolved.requested_identity.is_none());
}

#[test]
fn resolve_variant_query_preserves_empty_consequence_for_source_validation() {
    let resolved = resolve_variant_query(
        Some("BRAF".into()),
        None,
        Some("  ".into()),
        None,
        Vec::new(),
    )
    .unwrap();
    let VariantSearchPlan::Standard(resolved) = resolved else {
        panic!("expected standard search plan");
    };
    assert_eq!(resolved.consequence.as_deref(), Some(""));
}

#[test]
fn resolve_variant_query_maps_simple_gene_change_to_gene_and_hgvsp() {
    let resolved =
        resolve_variant_query(None, None, None, None, vec!["BRAF".into(), "V600E".into()]).unwrap();
    let VariantSearchPlan::Standard(resolved) = resolved else {
        panic!("expected standard search plan");
    };
    assert_eq!(resolved.gene.as_deref(), Some("BRAF"));
    assert_eq!(resolved.hgvsp.as_deref(), Some("V600E"));
    assert!(resolved.hgvsc.is_none());
    assert!(resolved.rsid.is_none());
    assert!(resolved.condition.is_none());
}

#[test]
fn resolve_variant_query_maps_long_form_positional_gene_change_to_gene_and_hgvsp() {
    let resolved = resolve_variant_query(
        None,
        None,
        None,
        None,
        vec!["BRAF".into(), "p.Val600Glu".into()],
    )
    .unwrap();
    let VariantSearchPlan::Standard(resolved) = resolved else {
        panic!("expected standard search plan");
    };
    assert_eq!(resolved.gene.as_deref(), Some("BRAF"));
    assert_eq!(resolved.hgvsp.as_deref(), Some("V600E"));
    assert!(resolved.hgvsc.is_none());
    assert!(resolved.rsid.is_none());
    assert!(resolved.condition.is_none());
    let requested = resolved
        .requested_identity
        .expect("exact positional identity");
    assert_eq!(requested.gene.as_deref(), Some("BRAF"));
    assert_eq!(requested.protein_change.as_deref(), Some("p.Val600Glu"));
}

#[test]
fn resolve_variant_query_preserves_complex_exact_protein_identity() {
    let resolved = resolve_variant_query(
        Some("EGFR".into()),
        Some("p.Glu746_Ala750del".into()),
        None,
        None,
        Vec::new(),
    )
    .unwrap();
    let VariantSearchPlan::Standard(resolved) = resolved else {
        panic!("expected standard search plan");
    };
    assert_eq!(resolved.hgvsp.as_deref(), Some("Glu746_Ala750del"));
    assert_eq!(
        resolved
            .requested_identity
            .and_then(|identity| identity.protein_change),
        Some("p.Glu746_Ala750del".into())
    );
}

#[test]
fn resolve_variant_query_maps_rsid_to_rsid_filter() {
    let resolved =
        resolve_variant_query(None, None, None, None, vec!["rs113488022".into()]).unwrap();
    let VariantSearchPlan::Standard(resolved) = resolved else {
        panic!("expected standard search plan");
    };
    assert_eq!(resolved.rsid.as_deref(), Some("rs113488022"));
    assert!(resolved.gene.is_none());
    assert!(resolved.hgvsp.is_none());
    assert!(resolved.hgvsc.is_none());
    assert!(resolved.condition.is_none());
    assert_eq!(
        resolved
            .requested_identity
            .and_then(|identity| identity.rsid),
        Some("rs113488022".into())
    );
}

#[test]
fn resolve_variant_query_preserves_conjunctive_rsid_and_protein_identity() {
    let resolved = resolve_variant_query(
        None,
        Some("p.Val600Glu".into()),
        None,
        None,
        vec!["rs113488022".into()],
    )
    .unwrap();
    let VariantSearchPlan::Standard(resolved) = resolved else {
        panic!("expected standard search plan");
    };
    assert_eq!(resolved.hgvsp.as_deref(), Some("V600E"));
    let requested = resolved.requested_identity.expect("conjunctive identity");
    assert_eq!(requested.rsid.as_deref(), Some("rs113488022"));
    assert_eq!(requested.protein_change.as_deref(), Some("p.Val600Glu"));
}

#[test]
fn resolve_variant_query_maps_gene_hgvsc_text_to_gene_and_hgvsc() {
    let resolved = resolve_variant_query(
        None,
        None,
        None,
        None,
        vec!["BRAF".into(), "c.1799T>A".into()],
    )
    .unwrap();
    let VariantSearchPlan::Standard(resolved) = resolved else {
        panic!("expected standard search plan");
    };
    assert_eq!(resolved.gene.as_deref(), Some("BRAF"));
    assert_eq!(resolved.hgvsc.as_deref(), Some("c.1799T>A"));
    assert!(resolved.hgvsp.is_none());
    assert!(resolved.rsid.is_none());
    assert!(resolved.condition.is_none());
    let requested = resolved.requested_identity.expect("exact coding identity");
    assert_eq!(requested.gene.as_deref(), Some("BRAF"));
    assert_eq!(requested.coding_change.as_deref(), Some("c.1799T>A"));
}

#[test]
fn resolve_variant_query_maps_exon_deletion_phrase_to_gene_and_consequence() {
    let resolved = resolve_variant_query(
        None,
        None,
        None,
        None,
        vec!["EGFR".into(), "Exon".into(), "19".into(), "Deletion".into()],
    )
    .unwrap();
    let VariantSearchPlan::Standard(resolved) = resolved else {
        panic!("expected standard search plan");
    };
    assert_eq!(resolved.gene.as_deref(), Some("EGFR"));
    assert_eq!(resolved.consequence.as_deref(), Some("inframe_deletion"));
    assert!(resolved.hgvsp.is_none());
    assert!(resolved.hgvsc.is_none());
    assert!(resolved.rsid.is_none());
    assert!(resolved.condition.is_none());
}

#[test]
fn resolve_variant_query_maps_gene_residue_alias_to_residue_alias_search() {
    let resolved =
        resolve_variant_query(None, None, None, None, vec!["PTPN22".into(), "620W".into()])
            .unwrap();
    let VariantSearchPlan::Standard(resolved) = resolved else {
        panic!("expected standard search plan");
    };
    assert_eq!(resolved.gene.as_deref(), Some("PTPN22"));
    assert_eq!(
        resolved.protein_alias,
        Some(crate::entities::variant::VariantProteinAlias {
            position: 620,
            residue: 'W',
        })
    );
    assert!(resolved.hgvsp.is_none());
    assert!(resolved.condition.is_none());
    assert!(resolved.requested_identity.is_none());
}

#[test]
fn resolve_variant_query_maps_gene_flag_residue_alias_to_residue_alias_search() {
    let resolved =
        resolve_variant_query(Some("PTPN22".into()), None, None, None, vec!["620W".into()])
            .unwrap();
    let VariantSearchPlan::Standard(resolved) = resolved else {
        panic!("expected standard search plan");
    };
    assert_eq!(resolved.gene.as_deref(), Some("PTPN22"));
    assert_eq!(
        resolved.protein_alias,
        Some(crate::entities::variant::VariantProteinAlias {
            position: 620,
            residue: 'W',
        })
    );
    assert!(resolved.hgvsp.is_none());
    assert!(resolved.condition.is_none());
}

#[test]
fn resolve_variant_query_uses_gene_context_for_standalone_protein_change() {
    let resolved = resolve_variant_query(
        Some("PTPN22".into()),
        None,
        None,
        None,
        vec!["R620W".into()],
    )
    .unwrap();
    let VariantSearchPlan::Standard(resolved) = resolved else {
        panic!("expected standard search plan");
    };
    assert_eq!(resolved.gene.as_deref(), Some("PTPN22"));
    assert_eq!(resolved.hgvsp.as_deref(), Some("R620W"));
    assert!(resolved.protein_alias.is_none());
}

#[test]
fn resolve_variant_query_uses_gene_context_for_long_form_single_token_change() {
    let resolved = resolve_variant_query(
        Some("BRAF".into()),
        None,
        None,
        None,
        vec!["p.Val600Glu".into()],
    )
    .unwrap();
    let VariantSearchPlan::Standard(resolved) = resolved else {
        panic!("expected standard search plan");
    };
    assert_eq!(resolved.gene.as_deref(), Some("BRAF"));
    assert_eq!(resolved.hgvsp.as_deref(), Some("V600E"));
    assert!(resolved.protein_alias.is_none());
}

#[test]
fn resolve_variant_query_returns_guidance_for_standalone_protein_change() {
    let resolved = resolve_variant_query(None, None, None, None, vec!["R620W".into()]).unwrap();
    let VariantSearchPlan::Guidance(guidance) = resolved else {
        panic!("expected guidance plan");
    };
    assert_eq!(guidance.query, "R620W");
    assert!(matches!(
        guidance.kind,
        crate::entities::variant::VariantGuidanceKind::ProteinChangeOnly { .. }
    ));
}

#[test]
fn resolve_variant_query_returns_guidance_for_long_form_single_token_change() {
    let resolved =
        resolve_variant_query(None, None, None, None, vec!["p.Val600Glu".into()]).unwrap();
    let VariantSearchPlan::Guidance(guidance) = resolved else {
        panic!("expected guidance plan");
    };
    assert_eq!(guidance.query, "p.Val600Glu");
    assert!(matches!(
        guidance.kind,
        crate::entities::variant::VariantGuidanceKind::ProteinChangeOnly { .. }
    ));
    assert_eq!(
        guidance.next_commands.first().map(String::as_str),
        Some("biomcp search variant --hgvsp V600E --limit 10")
    );
}

#[test]
fn resolve_variant_query_normalizes_long_form_hgvsp_flag() {
    let resolved = resolve_variant_query(
        Some("BRAF".into()),
        Some("p.Val600Glu".into()),
        None,
        None,
        Vec::new(),
    )
    .unwrap();
    let VariantSearchPlan::Standard(resolved) = resolved else {
        panic!("expected standard search plan");
    };
    assert_eq!(resolved.gene.as_deref(), Some("BRAF"));
    assert_eq!(resolved.hgvsp.as_deref(), Some("V600E"));
    assert!(resolved.hgvsc.is_none());
    assert!(resolved.rsid.is_none());
    assert!(resolved.condition.is_none());
    assert_eq!(
        resolved
            .requested_identity
            .and_then(|identity| identity.protein_change),
        Some("p.Val600Glu".into())
    );
}

#[test]
fn resolve_variant_query_preserves_stop_x_for_hgvsp_flag() {
    let resolved = resolve_variant_query(
        Some("PLN".into()),
        Some("L39X".into()),
        None,
        None,
        Vec::new(),
    )
    .unwrap();
    let VariantSearchPlan::Standard(resolved) = resolved else {
        panic!("expected standard search plan");
    };
    assert_eq!(resolved.gene.as_deref(), Some("PLN"));
    assert_eq!(resolved.hgvsp.as_deref(), Some("L39X"));
}

#[test]
fn resolve_variant_query_rejects_conflicts_with_positional_mapping() {
    let gene_conflict = resolve_variant_query(
        Some("TP53".into()),
        None,
        None,
        None,
        vec!["BRAF".into(), "V600E".into()],
    )
    .unwrap_err();
    assert!(format!("{gene_conflict}").contains("conflicts with --gene"));

    let hgvsp_conflict = resolve_variant_query(
        None,
        Some("G12D".into()),
        None,
        None,
        vec!["KRAS".into(), "G12C".into()],
    )
    .unwrap_err();
    assert!(format!("{hgvsp_conflict}").contains("conflicts with --hgvsp"));

    let consequence_conflict = resolve_variant_query(
        None,
        None,
        Some("missense_variant".into()),
        None,
        vec!["EGFR".into(), "Exon".into(), "19".into(), "Deletion".into()],
    )
    .unwrap_err();
    assert!(
        format!("{consequence_conflict}")
            .contains("Positional exon-deletion query conflicts with --consequence")
    );
}

#[tokio::test]
async fn variant_get_shorthand_json_returns_variant_guidance_metadata() {
    let cli = Cli::try_parse_from(["biomcp", "--json", "get", "variant", "R620W"]).expect("parse");
    let outcome = run_outcome(cli).await.expect("variant guidance outcome");

    assert_eq!(outcome.stream, OutputStream::Stdout);
    assert_eq!(outcome.exit_code, 1);

    let value: serde_json::Value =
        serde_json::from_str(&outcome.text).expect("valid variant guidance json");
    assert_eq!(
        value["_meta"]["alias_resolution"]["requested_entity"],
        "variant"
    );
    assert_eq!(
        value["_meta"]["alias_resolution"]["kind"],
        "protein_change_only"
    );
    assert_eq!(value["_meta"]["alias_resolution"]["query"], "R620W");
    assert_eq!(value["_meta"]["alias_resolution"]["change"], "R620W");
    assert_eq!(
        value["_meta"]["next_commands"][0],
        "biomcp search variant --hgvsp R620W --limit 10"
    );
}

#[tokio::test]
async fn variant_search_shorthand_json_returns_variant_guidance_metadata() {
    let cli =
        Cli::try_parse_from(["biomcp", "--json", "search", "variant", "R620W"]).expect("parse");
    let outcome = run_outcome(cli)
        .await
        .expect("variant search guidance outcome");

    assert_eq!(outcome.stream, OutputStream::Stdout);
    assert_eq!(outcome.exit_code, 1);

    let value: serde_json::Value =
        serde_json::from_str(&outcome.text).expect("valid variant guidance json");
    assert_eq!(
        value["_meta"]["alias_resolution"]["requested_entity"],
        "variant"
    );
    assert_eq!(
        value["_meta"]["alias_resolution"]["kind"],
        "protein_change_only"
    );
    assert_eq!(value["_meta"]["next_commands"][1], "biomcp discover R620W");
    assert_eq!(value["results"], serde_json::json!([]));
}

#[test]
fn ticket_377_variant_renderer_envelope_contracts() {
    let results = vec![crate::entities::variant::VariantSearchResult {
        id: "rs113488022".to_string(),
        genome_build: crate::entities::variant::GenomeBuild::Grch37,
        genome_build_provenance: "test".into(),
        gene: "BRAF".to_string(),
        hgvs_p: Some("p.V600E".to_string()),
        legacy_name: Some("BRAF V600E".to_string()),
        significance: Some("Pathogenic".to_string()),
        clinvar_stars: Some(3),
        gnomad_af: None,
        revel: Some(0.92),
        gerp: Some(5.1),
        source_identity: None,
        matched_alias: None,
    }];
    let next_commands = crate::render::markdown::search_next_commands_variant(
        &results,
        Some("BRAF"),
        Some("melanoma"),
    );
    let json = super::super::search_json_with_meta(
        results.clone(),
        crate::cli::PaginationMeta::offset(0, 1, 1, Some(1)),
        next_commands,
    )
    .expect("variant search_json_with_meta");
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid variant JSON");
    assert_eq!(value["results"][0]["genome_build"], "GRCh37");
    assert_eq!(value["results"][0]["genome_build_provenance"], "test");
    assert!(
        value["_meta"]["next_commands"]
            .as_array()
            .is_some_and(|commands| {
                commands
                    .iter()
                    .any(|command| command.as_str() == Some("biomcp get variant rs113488022"))
            })
    );

    let markdown = crate::render::markdown::variant_search_markdown_with_context(
        "gene=BRAF, condition=melanoma",
        &results,
        "",
        Some("BRAF"),
        Some("melanoma"),
    )
    .expect("variant markdown");
    assert!(markdown.contains("See also:"));
    assert!(markdown.contains("biomcp search disease --query melanoma"));

    let normalization = entity::VariantNormalizationResponse {
        input: "NM_004333.6:c.1799T>A".to_string(),
        services: vec![entity::VariantNormalizationAggregate::Legacy(
            entity::VariantNormalizationServiceResult {
                service: "mutalyzer".to_string(),
                status: entity::VariantNormalizationStatus::InvalidInput,
                input_description: Some("NM_004333.6:c.1799T>A".to_string()),
                normalized_description: None,
                corrected_description: None,
                transcript_description: None,
                protein: None,
                genomic_descriptions: vec![crate::entities::GenomicCoordinate {
                    coordinate: "NC_000007.14:g.140753336A>T".into(),
                    genome_build: "GRCh38".into(),
                    source: "test".into(),
                    provenance: None,
                }],
                warnings: vec!["fixture warning from normalization service".to_string()],
                message: Some("Invalid transcript HGVS".to_string()),
            },
        )],
    };
    let normalization_json = serde_json::to_value(&normalization).expect("normalization JSON");
    assert_eq!(
        normalization_json["services"][0]["genomic_descriptions"][0],
        serde_json::json!({
            "coordinate": "NC_000007.14:g.140753336A>T",
            "genome_build": "GRCh38",
            "source": "test"
        })
    );
    assert_eq!(normalization_json["services"][0]["status"], "invalid_input");
    assert_eq!(
        normalization_json["services"][0]["warnings"][0],
        "fixture warning from normalization service"
    );

    let normalization_markdown =
        crate::render::markdown::variant_normalization_markdown(&normalization);
    assert!(normalization_markdown.contains("Status: invalid_input"));
    assert!(normalization_markdown.contains("NC_000007.14:g.140753336A>T"));
    assert!(normalization_markdown.contains("fixture warning from normalization service"));
}
