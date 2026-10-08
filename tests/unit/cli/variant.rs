use clap::Parser;

#[path = "variant/articles.rs"]
mod articles;
#[path = "variant/parsing.rs"]
mod parsing;

use super::VariantSearchPlan;
use super::query::{
    GeneFirstNote, VariantQueryGeneRouting, apply_gene_first_routing, confirm_gene_first_candidate,
    gene_first_working_form, parse_simple_gene_change, resolve_variant_query,
    split_gene_first_candidate, split_leading_protein_change,
};

use crate::cli::{Cli, Commands, GetEntity, OutputStream, VariantCommand, run_outcome};
use crate::entities::variant as entity;
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

#[test]
fn erepo_help_labels_germline_scope_and_civic_destination() {
    let help = Cli::try_parse_from(["biomcp", "variant", "erepo", "--help"])
        .expect_err("help should stop parsing")
        .to_string();

    assert!(help.contains("germline"));
    assert!(help.contains("get gene <symbol> civic"));
    assert!(help.contains("get variant <id> civic"));
}

#[test]
fn erepo_gene_mode_is_bounded_and_mutually_exclusive() {
    let cli = Cli::try_parse_from([
        "biomcp", "variant", "erepo", "--gene", "PTEN", "--limit", "25", "--offset", "50",
    ])
    .expect("bounded gene search");
    match cli.command {
        Commands::Variant {
            cmd:
                VariantCommand::Erepo {
                    gene,
                    limit,
                    offset,
                    ..
                },
        } => {
            assert_eq!(gene.as_deref(), Some("PTEN"));
            assert_eq!(limit, 25);
            assert_eq!(offset, 50);
        }
        other => panic!("unexpected command: {other:?}"),
    }
    for args in [
        vec!["biomcp", "variant", "erepo", "CA1", "--gene", "PTEN"],
        vec![
            "biomcp", "variant", "erepo", "--gene", "PTEN", "--limit", "0",
        ],
        vec![
            "biomcp", "variant", "erepo", "--gene", "PTEN", "--limit", "101",
        ],
        vec![
            "biomcp", "variant", "erepo", "--gene", "PTEN", "--offset", "-1",
        ],
    ] {
        assert!(Cli::try_parse_from(args).is_err());
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
fn split_gene_first_candidate_accepts_symbol_shaped_first_tokens() {
    let cases = [
        ("SCN5A Brugada", Some(("SCN5A", "Brugada"))),
        ("TP53 osteosarcoma", Some(("TP53", "osteosarcoma"))),
        (
            "BRCA1 hereditary breast cancer",
            Some(("BRCA1", "hereditary breast cancer")),
        ),
        (
            "SCN5A   Brugada  syndrome",
            Some(("SCN5A", "Brugada syndrome")),
        ),
        // Single tokens keep the existing gene-only routing.
        ("SCN5A", None),
        // Lowercase words are not gene-shaped.
        ("Lung cancer", None),
        // Hyphenated symbols fall outside the exact-form token shape.
        ("H3-3A glioma", None),
        ("", None),
    ];
    for (query, expected) in cases {
        assert_eq!(
            split_gene_first_candidate(query),
            expected.map(|(gene, condition)| (gene.to_string(), condition.to_string())),
            "query: {query:?}"
        );
    }
}

#[test]
fn apply_gene_first_routing_routes_confirmed_symbols_and_refuses_the_rest() {
    let (resolved, note) = apply_gene_first_routing(
        "SCN5A".into(),
        None,
        "Brugada syndrome".into(),
        Some("SCN5A".into()),
        None,
        None,
    );
    assert_eq!(resolved.gene.as_deref(), Some("SCN5A"));
    assert_eq!(resolved.condition.as_deref(), Some("Brugada syndrome"));
    // A routed phrase carries its parsed form and a loosened alternative so a
    // zero-row search can say how it was read without repeating itself
    // (ticket 2022; the alternative drops the condition filter).
    assert_eq!(
        note.as_ref(),
        Some(&GeneFirstNote::Routed {
            parsed: "gene=SCN5A, condition=Brugada syndrome".into(),
            alternative: "biomcp search variant -g SCN5A".into(),
        })
    );

    // An uppercase non-gene first token such as BRUGADA is refused by the
    // oracle, so the whole phrase keeps the condition routing.
    let (resolved, note) =
        apply_gene_first_routing("BRUGADA".into(), None, "syndrome".into(), None, None, None);
    assert_eq!(resolved.gene, None);
    assert_eq!(resolved.condition.as_deref(), Some("BRUGADA syndrome"));
    assert_eq!(
        note,
        Some(GeneFirstNote::Refused {
            gene: "BRUGADA".into(),
            condition: "syndrome".into(),
        })
    );

    // Ticket 2022 keeps one oracle verdict shape: only an official symbol
    // confirms, and the confirmed symbol equals the routed token. The ERBB1
    // alias never reaches this branch because the oracle refuses it.
    let (resolved, _note) = apply_gene_first_routing(
        "EGFR".into(),
        None,
        "glioblastoma".into(),
        Some("EGFR".into()),
        None,
        None,
    );
    assert_eq!(resolved.gene.as_deref(), Some("EGFR"));
    assert_eq!(resolved.condition.as_deref(), Some("glioblastoma"));
}

#[test]
fn apply_gene_first_routing_moves_a_leading_protein_change_to_hgvsp() {
    let (resolved, note) = apply_gene_first_routing(
        "BRAF".into(),
        Some("V600E".into()),
        "melanoma".into(),
        Some("BRAF".into()),
        None,
        None,
    );
    assert_eq!(resolved.gene.as_deref(), Some("BRAF"));
    assert_eq!(resolved.hgvsp.as_deref(), Some("V600E"));
    assert_eq!(resolved.condition.as_deref(), Some("melanoma"));
    // An hgvsp filter claims the search identity, exactly as an explicit
    // --hgvsp flag does on the standard path.
    assert!(resolved.requested_identity.is_some());
    assert_eq!(
        note.as_ref(),
        Some(&GeneFirstNote::Routed {
            parsed: "gene=BRAF, hgvsp=V600E, condition=melanoma".into(),
            alternative: "biomcp search variant -g BRAF --hgvsp V600E".into(),
        })
    );

    // The refused branch keeps the whole phrase as the condition, protein
    // change included, and the explicit hgvsp flag survives.
    let (resolved, note) = apply_gene_first_routing(
        "BRUGADA".into(),
        Some("V600E".into()),
        "melanoma".into(),
        None,
        Some("p.Val600Glu".into()),
        None,
    );
    assert_eq!(resolved.gene, None);
    assert_eq!(
        resolved.condition.as_deref(),
        Some("BRUGADA V600E melanoma")
    );
    assert_eq!(resolved.hgvsp.as_deref(), Some("V600E"));
    assert_eq!(
        note,
        Some(GeneFirstNote::Refused {
            gene: "BRUGADA".into(),
            condition: "V600E melanoma".into(),
        })
    );
}

#[test]
fn apply_gene_first_routing_keeps_explicit_filters_on_both_paths() {
    // Confirmed path: the phrase routes gene-first and the explicit
    // consequence filter survives beside it.
    let (resolved, note) = apply_gene_first_routing(
        "SCN5A".into(),
        None,
        "Brugada".into(),
        Some("SCN5A".into()),
        None,
        Some("missense_variant".into()),
    );
    assert_eq!(resolved.gene.as_deref(), Some("SCN5A"));
    assert_eq!(resolved.condition.as_deref(), Some("Brugada"));
    assert_eq!(resolved.consequence.as_deref(), Some("missense_variant"));
    assert!(matches!(note.as_ref(), Some(GeneFirstNote::Routed { .. })));

    // Refused path: the whole phrase stays the condition and the explicit
    // hgvsp filter survives, normalized exactly as the standard path does.
    let (resolved, _note) = apply_gene_first_routing(
        "BRUGADA".into(),
        None,
        "syndrome".into(),
        None,
        Some("p.Val600Glu".into()),
        None,
    );
    assert_eq!(resolved.gene, None);
    assert_eq!(resolved.condition.as_deref(), Some("BRUGADA syndrome"));
    assert_eq!(resolved.hgvsp.as_deref(), Some("V600E"));
    // The standard path keeps the raw long form on the requested identity
    // while the filter takes the compact spelling.
    assert_eq!(
        resolved
            .requested_identity
            .and_then(|identity| identity.protein_change),
        Some("p.Val600Glu".into())
    );
}

#[test]
fn gene_first_working_form_quotes_multi_word_conditions() {
    assert_eq!(
        gene_first_working_form("SCN5A", None, "Brugada syndrome"),
        "biomcp search variant -g SCN5A --condition \"Brugada syndrome\""
    );
    assert_eq!(
        gene_first_working_form("BRAF", Some("V600E"), "melanoma"),
        "biomcp search variant -g BRAF --hgvsp V600E --condition melanoma"
    );
}

#[test]
fn split_leading_protein_change_takes_only_a_leading_change() {
    assert_eq!(
        split_leading_protein_change("V600E melanoma"),
        Some(("V600E".to_string(), "melanoma".to_string()))
    );
    assert_eq!(
        split_leading_protein_change("p.Val600Glu metastatic melanoma"),
        Some(("V600E".to_string(), "metastatic melanoma".to_string()))
    );
    // A condition that merely starts with letters is not a protein change.
    assert_eq!(split_leading_protein_change("liver cancer"), None);
    // A remainder that is only the protein change belongs to the exact
    // "GENE CHANGE" form, not the gene-first split.
    assert_eq!(split_leading_protein_change("V600E"), None);
}

#[test]
fn variant_query_gene_routing_preference_defaults_to_mygene() {
    let cases = [
        (None, VariantQueryGeneRouting::Mygene),
        (Some(""), VariantQueryGeneRouting::Mygene),
        (Some("mygene"), VariantQueryGeneRouting::Mygene),
        (Some("MYGENE"), VariantQueryGeneRouting::Mygene),
        (Some("off"), VariantQueryGeneRouting::Off),
        (Some("bogus"), VariantQueryGeneRouting::Mygene),
    ];
    for (value, expected) in cases {
        assert_eq!(
            VariantQueryGeneRouting::from_env_value(value),
            expected,
            "value: {value:?}"
        );
    }
}

/// Restores routing-related environment variables after one test; every
/// setter shares the `variant_routing_env` serial group.
struct RoutingEnvRestore(Vec<(&'static str, Option<std::ffi::OsString>)>);

impl RoutingEnvRestore {
    fn set(values: &[(&'static str, &str)]) -> Self {
        let prior = values
            .iter()
            .map(|(name, _)| (*name, std::env::var_os(name)))
            .collect::<Vec<_>>();
        // SAFETY: these tests share the variant_routing_env serial group, and
        // the guard restores every value before releasing that group.
        unsafe {
            for (name, value) in values {
                std::env::set_var(name, value);
            }
        }
        Self(prior)
    }
}

impl Drop for RoutingEnvRestore {
    fn drop(&mut self) {
        // SAFETY: see RoutingEnvRestore::set.
        unsafe {
            for (name, value) in self.0.iter() {
                match value {
                    Some(value) => std::env::set_var(name, value),
                    None => std::env::remove_var(name),
                }
            }
        }
    }
}

#[tokio::test]
#[serial_test::serial(variant_routing_env)]
async fn gene_routing_preference_off_skips_the_mygene_request() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind local mygene listener");
    let base = format!("http://{}", listener.local_addr().unwrap());
    let connections = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter = std::sync::Arc::clone(&connections);
    let server = tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            drop(stream);
            counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    });
    let _env = RoutingEnvRestore::set(&[
        ("BIOMCP_MYGENE_BASE", &base),
        ("BIOMCP_VARIANT_QUERY_GENE_ROUTING", "off"),
    ]);
    let confirmed = confirm_gene_first_candidate("SCN5A").await;
    server.abort();
    assert_eq!(confirmed, None);
    assert_eq!(
        connections.load(std::sync::atomic::Ordering::SeqCst),
        0,
        "preference off must not contact MyGene at all"
    );
}

#[test]
fn resolve_variant_query_offers_gene_first_candidate_for_free_text_phrases() {
    let resolved = resolve_variant_query(
        None,
        None,
        None,
        None,
        vec!["SCN5A".into(), "Brugada".into()],
    )
    .unwrap();
    assert_eq!(
        resolved,
        VariantSearchPlan::GeneFirstCandidate {
            gene: "SCN5A".into(),
            protein_change: None,
            condition: "Brugada".into(),
            hgvsp: None,
            consequence: None,
        }
    );

    // Explicit flags ride on the candidate so neither oracle branch can
    // drop them.
    let resolved = resolve_variant_query(
        None,
        Some("  p.Val600Glu  ".into()),
        Some("missense_variant".into()),
        None,
        vec!["SCN5A".into(), "Brugada".into()],
    )
    .unwrap();
    assert_eq!(
        resolved,
        VariantSearchPlan::GeneFirstCandidate {
            gene: "SCN5A".into(),
            protein_change: None,
            condition: "Brugada".into(),
            hgvsp: Some("p.Val600Glu".into()),
            consequence: Some("missense_variant".into()),
        }
    );
}

#[test]
fn resolve_variant_query_splits_a_protein_change_after_the_gene() {
    let plan = resolve_variant_query(
        None,
        None,
        None,
        None,
        vec!["BRAF".into(), "V600E".into(), "melanoma".into()],
    )
    .unwrap();
    assert_eq!(
        plan,
        VariantSearchPlan::GeneFirstCandidate {
            gene: "BRAF".into(),
            protein_change: Some("V600E".into()),
            condition: "melanoma".into(),
            hgvsp: None,
            consequence: None,
        }
    );

    // The leftover protein change conflicts with an explicit --hgvsp flag,
    // exactly as the other positional protein-change forms do.
    let err = resolve_variant_query(
        None,
        Some("V600K".into()),
        None,
        None,
        vec!["BRAF".into(), "V600E".into(), "melanoma".into()],
    )
    .unwrap_err();
    assert!(err.to_string().contains("conflicts with --hgvsp"));
}

#[test]
fn resolve_variant_query_keeps_non_gene_phrases_as_conditions() {
    let resolved = resolve_variant_query(
        None,
        None,
        None,
        None,
        vec!["melanoma".into(), "treatment".into()],
    )
    .unwrap();
    let VariantSearchPlan::Standard(resolved) = resolved else {
        panic!("expected standard search plan");
    };
    assert_eq!(resolved.gene, None);
    assert_eq!(resolved.condition.as_deref(), Some("melanoma treatment"));
}

#[test]
fn resolve_variant_query_keeps_explicit_gene_flag_routing() {
    let resolved = resolve_variant_query(
        Some("BRAF".into()),
        None,
        None,
        None,
        vec!["SCN5A".into(), "Brugada".into()],
    )
    .unwrap();
    let VariantSearchPlan::Standard(resolved) = resolved else {
        panic!("expected standard search plan");
    };
    assert_eq!(resolved.gene.as_deref(), Some("BRAF"));
    assert_eq!(resolved.condition.as_deref(), Some("SCN5A Brugada"));
}

#[test]
fn resolve_variant_query_still_rejects_condition_flag_with_positional_phrase() {
    let error = resolve_variant_query(
        None,
        None,
        None,
        Some("Brugada".into()),
        vec!["SCN5A".into(), "Brugada".into()],
    )
    .unwrap_err();
    assert!(format!("{error}").contains("not both"));
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
    let results = vec![entity::VariantSearchResult {
        id: "rs113488022".to_string(),
        genome_build: entity::GenomeBuild::Grch37,
        genome_build_provenance: "test".into(),
        gene: "BRAF".to_string(),
        hgvs_p: Some("p.V600E".to_string()),
        hgvs_c: Some("c.1799T>A".to_string()),
        transcript: Some("NM_004333.6".to_string()),
        legacy_name: Some("BRAF V600E".to_string()),
        significance: Some("Pathogenic".to_string()),
        significance_source: None,
        significance_evaluated: None,
        clinvar_stars: Some(3),
        gnomad_af: None,
        revel: Some(0.92),
        gerp: Some(5.1),
        source_identity: None,
        matched_alias: None,
        transcript_annotations_complete: None,
        transcript_annotations: None,
    }];
    let next_commands = crate::render::markdown::search_next_commands_variant(
        &results,
        Some("BRAF"),
        Some("melanoma"),
    );
    let json = crate::cli::search_json_with_meta(
        results.clone(),
        crate::cli::PaginationMeta::offset(0, 1, 1, Some(1)),
        next_commands,
    )
    .expect("variant search_json_with_meta");
    let json = crate::render::json::with_variant_search_resolution(
        json,
        None,
        None,
        Default::default(),
        Vec::new(),
    )
    .expect("variant search resolution envelope");
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid variant JSON");
    assert_eq!(value["filter_evaluation"], serde_json::json!({}));
    assert!(value.get("filter_resolution").is_none());
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
        &Default::default(),
        &[],
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

#[test]
fn variant_filter_evaluation_renderers_keep_mixed_states_and_order() {
    let evaluation = std::collections::BTreeMap::from([
        ("hgvsp", entity::VariantFilterEvaluationStatus::Evaluated),
        ("gene", entity::VariantFilterEvaluationStatus::Unavailable),
    ]);
    let markdown = crate::render::markdown::variant_search_markdown_with_context(
        "gene=MISSING, hgvsp=p.V1A",
        &[],
        "",
        Some("MISSING"),
        None,
        &evaluation,
        &[],
    )
    .expect("mixed filter evaluation markdown");
    let gene = markdown.find("gene: unavailable").expect("gene evaluation");
    let hgvsp = markdown.find("hgvsp: evaluated").expect("hgvsp evaluation");
    assert!(markdown.contains("## Filter evaluation"));
    assert!(!markdown.contains("Filter resolution"));
    assert!(gene < hgvsp, "BTreeMap order must reach Markdown");

    let json = crate::cli::search_json_with_meta(
        Vec::<entity::VariantSearchResult>::new(),
        crate::cli::PaginationMeta::offset(0, 1, 0, Some(0)),
        Vec::new(),
    )
    .expect("empty variant search envelope");
    let json = crate::render::json::with_variant_search_resolution(
        json,
        None,
        None,
        evaluation,
        Vec::new(),
    )
    .expect("mixed filter evaluation JSON");
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid variant JSON");
    assert_eq!(
        value["filter_evaluation"],
        serde_json::json!({"gene": "unavailable", "hgvsp": "evaluated"})
    );
    assert!(value.get("filter_resolution").is_none());
    assert!(value.get("requested_variant").is_none());
    assert!(value.get("resolution").is_none());
}
