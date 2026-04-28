//! Top-level CLI payloads and subcommands that stay outside the per-entity families.

use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct HealthArgs {
    /// Check external APIs only
    #[arg(long)]
    pub apis_only: bool,
}

#[derive(Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmaCommand {
    /// Force refresh the EMA local data feeds
    Sync,
}

#[derive(Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhoCommand {
    /// Force refresh the WHO Prequalification local exports (finished pharma, API, vaccines)
    Sync,
}

#[derive(Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CvxCommand {
    /// Force refresh the CDC CVX/MVX vaccine identity bundle
    Sync,
}

#[derive(Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DdinterCommand {
    /// Force refresh the eight DDInter CSV files
    Sync,
}

#[derive(Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GtrCommand {
    /// Force refresh the local NCBI GTR diagnostic bundle
    Sync,
}

#[derive(Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhoIvdCommand {
    /// Force refresh the WHO Prequalified IVD diagnostic CSV export
    Sync,
}

#[derive(Args, Debug)]
pub struct ServeHttpArgs {
    /// Host address to bind
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
    /// Port to listen on
    #[arg(long, default_value = "8080")]
    pub port: u16,
}

#[derive(Args, Debug)]
pub struct UpdateArgs {
    /// Check for updates, but do not install
    #[arg(long)]
    pub check: bool,
    /// UNSAFE: install even when the release SHA256 checksum sidecar is missing
    #[arg(long = "allow-missing-checksum")]
    pub allow_missing_checksum: bool,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Optional entity name (gene, variant, article, trial, diagnostic, drug, disease, pgx, gwas, pathway, protein, study, adverse-event, search-all, suggest)
    pub entity: Option<String>,
}

#[derive(Args, Debug)]
pub struct BatchArgs {
    /// Entity type (gene, variant, article, trial, drug, disease, pgx, pathway, protein, adverse-event)
    pub entity: String,
    /// Comma-separated IDs (max 10)
    pub ids: String,
    /// Optional comma-separated sections to request on each get call
    #[arg(long)]
    pub sections: Option<String>,
    /// Trial source when entity=trial (ctgov or nci)
    #[arg(long, default_value = "ctgov")]
    pub source: String,
}

#[derive(Args, Debug)]
pub struct EnrichArgs {
    /// Comma-separated HGNC symbols (e.g., BRAF,KRAS,NRAS)
    pub genes: String,
    /// Maximum enrichment terms (default: 10)
    #[arg(short, long, default_value = "10")]
    pub limit: usize,
}

#[derive(Args, Debug)]
pub struct DiscoverArgs {
    /// Free-text biomedical query
    pub query: String,
}

#[derive(Args, Debug)]
pub struct SuggestArgs {
    /// Free-text biomedical question to route to a BioMCP playbook
    pub question: String,
}

#[derive(Args, Debug)]
pub struct VersionArgs {
    /// Include executable provenance and PATH diagnostics
    #[arg(long)]
    pub verbose: bool,
}

mod dispatch;
pub(crate) use self::dispatch::{
    handle_batch, handle_cvx, handle_ddinter, handle_ema, handle_enrich, handle_gtr,
    handle_uninstall, handle_version, handle_who, handle_who_ivd,
};

#[cfg(test)]
mod tests;
