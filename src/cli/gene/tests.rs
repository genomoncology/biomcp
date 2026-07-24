use clap::{CommandFactory, Parser};

use super::{CspecCommand, GeneCommand};
use crate::cli::{Cli, Commands, OutputStream};
use crate::entities::discover::{
    AliasCanonicalMatch, AliasFallbackDecision, DiscoverConfidence, DiscoverType, MatchTier,
};

fn render_gene_get_long_help() -> String {
    let mut command = Cli::command();
    let get = command
        .find_subcommand_mut("get")
        .expect("get subcommand should exist");
    let gene = get
        .find_subcommand_mut("gene")
        .expect("gene get subcommand should exist");
    let mut help = Vec::new();
    gene.write_long_help(&mut help)
        .expect("gene help should render");
    String::from_utf8(help).expect("help should be utf-8")
}

#[test]
fn get_gene_help_includes_when_to_use_guidance() {
    let help = render_gene_get_long_help();

    assert!(help.contains("When to use:"));
    assert!(help.contains("default card"));
    assert!(help.contains("protein, hpa, expression, diseases, diagnostics, or funding"));
    assert!(help.contains("BRCA1 diagnostics"));
    assert!(help.contains("ERBB2 funding"));
}

#[test]
fn gene_get_alias_parses_as_definition_subcommand() {
    let cli = Cli::try_parse_from(["biomcp", "gene", "get", "BRAF"])
        .expect("gene get alias should parse");

    match cli.command {
        Commands::Gene {
            cmd: GeneCommand::Definition { symbol },
        } => assert_eq!(symbol, "BRAF"),
        other => panic!("unexpected command: {other:?}"),
    }
}

#[test]
fn gene_cspec_document_parses_as_nested_raw_subcommand() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "gene",
        "cspec",
        "document",
        "capture:cspec:sha256:x",
    ])
    .expect("nested CSpec document command should parse");

    match cli.command {
        Commands::Gene {
            cmd: GeneCommand::Cspec(args),
        } => match args.command {
            Some(CspecCommand::Document { capture_id }) => {
                assert_eq!(capture_id, "capture:cspec:sha256:x");
            }
            other => panic!("unexpected CSpec command: {other:?}"),
        },
        other => panic!("unexpected command: {other:?}"),
    }
}

#[test]
fn gene_bare_symbol_parses_as_external_subcommand() {
    let cli =
        Cli::try_parse_from(["biomcp", "gene", "BRAF"]).expect("bare gene symbol should parse");

    match cli.command {
        Commands::Gene {
            cmd: GeneCommand::External(args),
        } => assert_eq!(args, vec!["BRAF"]),
        other => panic!("unexpected command: {other:?}"),
    }
}

#[test]
fn gene_pathways_parses_limit_and_offset() {
    let cli = Cli::try_parse_from([
        "biomcp", "gene", "pathways", "BRAF", "--limit", "5", "--offset", "1",
    ])
    .expect("gene pathways pagination flags should parse");

    match cli.command {
        Commands::Gene {
            cmd:
                GeneCommand::Pathways {
                    symbol,
                    limit,
                    offset,
                },
        } => {
            assert_eq!(symbol, "BRAF");
            assert_eq!(limit, 5);
            assert_eq!(offset, 1);
        }
        other => panic!("unexpected command: {other:?}"),
    }
}

#[test]
fn handle_get_gene_alias_fallback_returns_markdown_suggestion() {
    let decision = AliasFallbackDecision::Canonical(AliasCanonicalMatch {
        requested_entity: DiscoverType::Gene,
        query: "ERBB1".to_string(),
        canonical: "EGFR".to_string(),
        canonical_id: "HGNC:3236".to_string(),
        confidence: DiscoverConfidence::CanonicalId,
        match_tier: MatchTier::Exact,
        sources: vec!["OLS4".to_string()],
        next_commands: vec!["biomcp get gene EGFR".to_string()],
    });
    let outcome =
        super::super::alias_suggestion_outcome("ERBB1", DiscoverType::Gene, &decision, false)
            .expect("alias outcome");

    assert_eq!(outcome.stream, OutputStream::Stderr);
    assert_eq!(outcome.exit_code, 1);
    assert!(outcome.text.contains("Error: gene 'ERBB1' not found."));
    assert!(
        outcome
            .text
            .contains("Did you mean: `biomcp get gene EGFR`")
    );
}
