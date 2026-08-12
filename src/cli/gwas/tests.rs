use clap::Parser;

use crate::cli::{Cli, Commands, SearchEntity};

#[test]
fn search_gwas_parses_positional_query() {
    let cli = Cli::try_parse_from(["biomcp", "search", "gwas", "BRAF", "--limit", "2"])
        .expect("search gwas should parse");

    let Cli {
        command:
            Commands::Search {
                entity:
                    SearchEntity::Gwas(crate::cli::gwas::GwasSearchArgs {
                        gene,
                        positional_query,
                        trait_query,
                        p_value,
                        limit,
                        offset,
                    }),
            },
        ..
    } = cli
    else {
        panic!("expected search gwas command");
    };

    assert_eq!(gene, None);
    assert_eq!(positional_query.as_deref(), Some("BRAF"));
    assert_eq!(trait_query, None);
    assert_eq!(p_value, None);
    assert_eq!(limit, 2);
    assert_eq!(offset, 0);
}

#[test]
fn search_args_validate_probability_threshold_before_backend_lookup() {
    for value in ["NaN", "+inf", "-inf", "1e309", "0", "-0.01", "1.01"] {
        let cli = Cli::try_parse_from([
            "biomcp",
            "search",
            "gwas",
            "BRAF",
            &format!("--p-value={value}"),
        ])
        .expect("floating-point threshold should parse");
        let Cli {
            command:
                Commands::Search {
                    entity: SearchEntity::Gwas(args),
                },
            ..
        } = cli
        else {
            panic!("expected search gwas command");
        };

        let err = super::dispatch::validate_search_args(&args)
            .expect_err("invalid p-value should fail before backend lookup");
        assert!(matches!(
            &err,
            crate::error::BioMcpError::InvalidArgument(_)
        ));
        assert!(err.to_string().contains("--p-value"));
    }

    for value in ["5e-8", "1"] {
        let cli = Cli::try_parse_from([
            "biomcp",
            "search",
            "gwas",
            "BRAF",
            &format!("--p-value={value}"),
        ])
        .expect("valid p-value should parse");
        let Cli {
            command:
                Commands::Search {
                    entity: SearchEntity::Gwas(args),
                },
            ..
        } = cli
        else {
            panic!("expected search gwas command");
        };
        super::dispatch::validate_search_args(&args).expect("valid p-value should pass");
    }
}

#[test]
fn search_args_reject_zero_limit_before_backend_lookup() {
    let cli = Cli::try_parse_from(["biomcp", "search", "gwas", "BRAF", "--limit", "0"])
        .expect("search gwas should parse");

    let Cli {
        command: Commands::Search {
            entity: SearchEntity::Gwas(args),
        },
        json,
        ..
    } = cli
    else {
        panic!("expected search gwas command");
    };

    assert!(!json);
    let err =
        super::dispatch::validate_search_args(&args).expect_err("zero gwas limit should fail fast");
    assert!(err.to_string().contains("--limit must be between 1 and 50"));
}

#[test]
fn search_args_reject_windows_beyond_the_provider_budget_without_panicking() {
    for (limit, offset) in [(1, 50), (1, 200), (1, usize::MAX), (2, 49)] {
        let args = crate::cli::gwas::GwasSearchArgs {
            gene: Some("BRAF".into()),
            positional_query: None,
            trait_query: None,
            p_value: None,
            limit,
            offset,
        };
        let err = super::dispatch::validate_search_args(&args)
            .expect_err("GWAS windows beyond 50 must fail before transport");
        assert!(err.to_string().contains("--offset + --limit must be <= 50"));
    }

    for (limit, offset) in [(1, 0), (1, 49), (50, 0)] {
        let args = crate::cli::gwas::GwasSearchArgs {
            gene: Some("BRAF".into()),
            positional_query: None,
            trait_query: None,
            p_value: None,
            limit,
            offset,
        };
        super::dispatch::validate_search_args(&args).expect("window at or below 50 is valid");
    }
}

#[test]
fn region_is_not_an_advertised_or_accepted_gwas_filter() {
    let error = Cli::try_parse_from([
        "biomcp",
        "search",
        "gwas",
        "--region",
        "7:55000000-55200000",
    ])
    .expect_err("unsupported GWAS region filter must be rejected by the command boundary");
    assert!(error.to_string().contains("unexpected argument '--region'"));
}
