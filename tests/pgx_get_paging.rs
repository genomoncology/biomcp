use biomcp_cli::cli::{Cli, Commands, GetEntity};
use clap::Parser;

#[test]
fn get_pgx_accepts_named_sections_and_paging_in_normal_positions() {
    for argv in [
        vec![
            "biomcp",
            "get",
            "pgx",
            "CYP2D6",
            "recommendations",
            "--limit",
            "7",
            "--offset",
            "14",
        ],
        vec![
            "biomcp",
            "get",
            "pgx",
            "--limit",
            "7",
            "--offset",
            "14",
            "CYP2D6",
            "recommendations",
        ],
    ] {
        let cli = Cli::try_parse_from(argv).expect("get pgx paging");
        let Commands::Get {
            entity: GetEntity::Pgx(args),
        } = cli.command
        else {
            panic!("get pgx")
        };
        assert_eq!(args.sections, ["recommendations"]);
        assert_eq!((args.limit, args.offset, args.full), (7, 14, false));
    }
}

#[test]
fn get_pgx_accepts_interactions_and_full() {
    let cli = Cli::try_parse_from(["biomcp", "get", "pgx", "CYP2D6", "interactions"])
        .expect("interactions");
    let Commands::Get {
        entity: GetEntity::Pgx(args),
    } = cli.command
    else {
        panic!("get pgx")
    };
    assert_eq!(args.sections, ["interactions"]);
    let cli = Cli::try_parse_from(["biomcp", "get", "pgx", "CYP2D6", "--full"]).expect("full");
    let Commands::Get {
        entity: GetEntity::Pgx(args),
    } = cli.command
    else {
        panic!("get pgx")
    };
    assert!(args.full);
}
