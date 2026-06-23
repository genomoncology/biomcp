use super::*;

fn render_study_long_help() -> String {
    let mut command = Cli::command();
    let study = command
        .find_subcommand_mut("study")
        .expect("study subcommand should exist");
    let mut help = Vec::new();
    study
        .write_long_help(&mut help)
        .expect("study help should render");
    String::from_utf8(help).expect("help should be utf-8")
}

fn render_study_subcommand_long_help(name: &str) -> String {
    let mut command = Cli::command();
    let study = command
        .find_subcommand_mut("study")
        .expect("study subcommand should exist");
    let subcommand = study
        .find_subcommand_mut(name)
        .unwrap_or_else(|| panic!("study {name} subcommand should exist"));
    let mut help = Vec::new();
    subcommand
        .write_long_help(&mut help)
        .expect("study subcommand help should render");
    String::from_utf8(help).expect("help should be utf-8")
}

#[test]
fn study_help_lists_descriptions_for_all_subcommands() {
    let help = render_study_long_help();

    assert!(help.contains("List locally available cBioPortal studies"));
    assert!(help.contains("List downloadable study IDs or install a study locally"));
    assert!(help.contains("Run per-study gene query"));
    assert!(help.contains("Rank the most frequently mutated genes in a study"));
    assert!(
        help.contains(
            "Intersect sample filters across mutation, CNA, expression, and clinical data"
        )
    );
    assert!(help.contains("Split a cohort into mutant vs wildtype groups"));
    assert!(help.contains("Summarize KM survival and log-rank statistics by mutation group"));
    assert!(help.contains("Compare expression or mutation rate across mutation groups"));
    assert!(help.contains("Compute pairwise mutation co-occurrence across genes"));
}

#[test]
fn study_query_help_describes_key_flags_and_aliases() {
    let help = render_study_subcommand_long_help("query");

    assert!(help.contains("cBioPortal study ID"));
    assert!(help.contains("HGNC gene symbol to summarize"));
    assert!(help.contains("Canonical values: mutations, cna, expression, sv."));
    assert!(help.contains("copy_number, copy-number, expr, fusion"));
}

#[test]
fn study_top_mutated_help_describes_limit() {
    let help = render_study_subcommand_long_help("top-mutated");

    assert!(help.contains("cBioPortal study ID"));
    assert!(help.contains("Maximum number of genes to display, 1-50 (default: 10)"));
}

#[test]
fn study_filter_help_describes_each_filter_flag() {
    let help = render_study_subcommand_long_help("filter");

    assert!(help.contains("Keep samples with a mutation in GENE"));
    assert!(help.contains("Keep samples with high-level copy-number amplification in GENE"));
    assert!(help.contains("Keep samples with a deep copy-number deletion in GENE"));
    assert!(help.contains("Keep samples where GENE expression exceeds THRESHOLD"));
    assert!(help.contains("Keep samples where GENE expression is below THRESHOLD"));
    assert!(help.contains("format: GENE:THRESHOLD; repeatable"));
    assert!(help.contains("Keep samples matching this cancer type label"));
}

#[test]
fn study_cohort_help_describes_gene_split() {
    let help = render_study_subcommand_long_help("cohort");

    assert!(help.contains("cBioPortal study ID"));
    assert!(help.contains("HGNC gene symbol used to split mutant vs wildtype groups"));
}

#[test]
fn study_survival_help_describes_endpoint_values_and_aliases() {
    let help = render_study_subcommand_long_help("survival");

    assert!(help.contains("cBioPortal study ID"));
    assert!(help.contains("HGNC gene symbol used to define mutant vs wildtype groups"));
    assert!(help.contains("Canonical values: os, dfs, pfs, dss."));
    assert!(help.contains("Accepted aliases: overall, overall_survival,"));
    assert!(help.contains("disease_free, progression_free, disease_specific"));
}

#[test]
fn study_compare_help_describes_type_and_target() {
    let help = render_study_subcommand_long_help("compare");

    assert!(help.contains("cBioPortal study ID"));
    assert!(help.contains("HGNC gene symbol used to define mutant vs wildtype groups"));
    assert!(help.contains("Canonical values: expression, mutations."));
    assert!(help.contains("Accepted aliases: expr, mutation"));
    assert!(help.contains("Target gene symbol to compare across mutation groups"));
}

#[test]
fn study_co_occurrence_help_describes_gene_list_contract() {
    let help = render_study_subcommand_long_help("co-occurrence");

    assert!(help.contains("cBioPortal study ID"));
    assert!(help.contains("Comma-separated HGNC gene symbols"));
    assert!(help.contains("2-10 genes"));
}

#[test]
fn study_download_help_describes_list_and_study_id() {
    let help = render_study_subcommand_long_help("download");

    assert!(help.contains("[STUDY_ID]"));
    assert!(help.contains("List available remote study IDs"));
    assert!(help.contains("Study ID to download (required unless --list;"));
}
