use clap::{CommandFactory, Parser};

use crate::cli::{Cli, Commands, DrugRegionArg, GetEntity, SearchEntity};

fn subcommand_help(parent: &str, child: &str) -> String {
    let mut command = Cli::command();
    let parent = command
        .find_subcommand_mut(parent)
        .expect("parent subcommand should exist");
    let child = parent
        .find_subcommand_mut(child)
        .expect("child subcommand should exist");
    let mut help = Vec::new();
    child
        .write_long_help(&mut help)
        .expect("subcommand help should render");
    String::from_utf8(help).expect("help should be utf-8")
}

#[test]
fn public_region_aliases_are_aligned_across_parser_help_list_and_docs() {
    let search_cli = Cli::try_parse_from([
        "biomcp", "search", "drug", "aspirin", "--region", "ema", "--limit", "1",
    ])
    .expect("search drug ema alias should parse");
    let search_region = match search_cli.command {
        Commands::Search {
            entity: SearchEntity::Drug(super::DrugSearchArgs { region, .. }),
        } => region,
        other => panic!("unexpected command: {other:?}"),
    };
    assert_eq!(search_region, Some(DrugRegionArg::Eu));

    let get_cli = Cli::try_parse_from([
        "biomcp",
        "get",
        "drug",
        "Dupixent",
        "regulatory",
        "--region",
        "ema",
    ])
    .expect("get drug ema alias should parse");
    let get_region = match get_cli.command {
        Commands::Get {
            entity: GetEntity::Drug(super::DrugGetArgs { region, .. }),
        } => region,
        other => panic!("unexpected command: {other:?}"),
    };
    assert_eq!(get_region, Some(DrugRegionArg::Eu));

    let search_help = subcommand_help("search", "drug");
    assert!(search_help.contains("--region ema"));
    assert!(search_help.contains("canonical --region eu"));

    let get_help = subcommand_help("get", "drug");
    assert!(get_help.contains("--region ema"));
    assert!(get_help.contains("canonical `eu` region value"));

    let list = crate::cli::list::render(Some("drug")).expect("list drug should render");
    assert!(list.contains("search drug <query> --region <us|eu|ema|who|all>"));
    assert!(list.contains("get drug <name> regulatory [--region <us|eu|ema|who|all>]"));
    assert!(list.contains("`ema` is accepted as an input alias"));
    assert!(list.contains("canonical `eu` drug region value"));

    let user_guide = include_str!("../../../docs/user-guide/cli-reference.md");
    assert!(user_guide.contains("--region ema"));
    assert!(user_guide.contains("canonical `eu` region value"));

    let ux_reference = include_str!("../../../architecture/ux/cli-reference.md");
    assert!(ux_reference.contains("--region ema"));
    assert!(ux_reference.contains("--region eu"));
}
