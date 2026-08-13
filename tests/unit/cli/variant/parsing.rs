use clap::Parser;

use crate::cli::{Cli, Commands, SearchEntity};

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
