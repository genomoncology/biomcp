//! Top-level CLI payloads and subcommands that stay outside the per-entity families.

use clap::{Args, Subcommand, ValueEnum};

#[derive(Args, Debug)]
pub struct HealthArgs {
    /// Check external APIs only
    #[arg(long)]
    pub apis_only: bool,
    /// Check only this API (repeatable; names come from `biomcp health`)
    #[arg(long = "api", value_name = "CANONICAL_NAME")]
    pub apis: Vec<String>,
    /// Exit with status 1 after rendering when any probe is in the error bucket
    #[arg(long)]
    pub fail_on_error: bool,
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
pub enum GenCcCommand {
    /// Revalidate the local GenCC gene-disease validity dataset
    Sync,
}

#[derive(Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhoIvdCommand {
    /// Force refresh the WHO Prequalified IVD diagnostic CSV export
    Sync,
}

#[derive(Args, Debug)]
#[command(after_help = "POST /mcp accepts encoded request bodies up to 65,536 bytes inclusive.")]
pub struct ServeHttpArgs {
    /// Host address to bind
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
    /// Port to listen on
    #[arg(long, default_value = "8080", value_parser = parse_http_port)]
    pub port: u16,
    /// Hostnames or IP addresses to allow, optionally with ports (comma-separated)
    #[arg(long, value_delimiter = ',', conflicts_with = "unsafe_allow_any_host")]
    pub allowed_hosts: Vec<String>,
    /// Accept any Host header. This removes only the Host check; it does not add authentication or encryption.
    #[arg(long)]
    pub unsafe_allow_any_host: bool,
}

fn parse_http_port(value: &str) -> Result<u16, String> {
    let port = value
        .trim()
        .parse::<u16>()
        .map_err(|_| "--port must be between 1 and 65535".to_string())?;
    if port == 0 {
        return Err("--port must be between 1 and 65535".to_string());
    }
    Ok(port)
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpClient {
    Codex,
    ClaudeDesktop,
    ClaudeCode,
    Cursor,
    Cline,
    Vscode,
    Json,
}

#[derive(Args, Debug)]
pub struct McpConfigArgs {
    /// Print config for one client; omit to list supported clients
    #[arg(long, value_enum)]
    pub client: Option<McpClient>,
    /// Use the resolved absolute executable path instead of bare `biomcp`
    #[arg(long)]
    pub absolute_path: bool,
}

#[derive(Args, Debug)]
pub struct UpdateArgs {
    /// Check for updates, but do not install
    #[arg(long)]
    pub check: bool,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    #[arg(
        help = "Optional entity name. Canonical values:\n(gene, variant, article, author, trial, diagnostic, drug, disease, phenotype, pgx,\ngwas, pathway, protein, study, adverse-event, search-all, discover, batch, enrich, skill)"
    )]
    pub entity: Option<String>,
}

#[derive(Args, Debug)]
pub struct BatchArgs {
    /// Entity type (gene, variant, article, trial, drug, disease, pgx, pathway, protein, adverse-event)
    pub entity: String,
    /// Comma-separated IDs (max 10)
    pub ids: String,
    /// Optional comma-separated sections (not supported for adverse-event batches)
    #[arg(long)]
    pub sections: Option<String>,
    /// Trial source when entity=trial (ctgov or nci)
    #[arg(long)]
    pub source: Option<String>,
}

#[derive(Args, Debug)]
pub struct EnrichArgs {
    /// Comma-separated HGNC symbols (e.g., BRAF,KRAS,NRAS)
    pub genes: String,
    /// Maximum enrichment terms, 1-50 (default: 10)
    #[arg(short, long, default_value = "10")]
    pub limit: usize,
}

#[derive(Args, Debug)]
pub struct DiscoverArgs {
    /// Free-text biomedical query
    pub query: String,
    /// Maximum concepts returned (applied after validation and ranking)
    #[arg(long, default_value_t = 5)]
    pub limit: usize,
    /// Zero-based offset into the stable ranked concepts
    #[arg(long, default_value_t = 0)]
    pub offset: usize,
    /// Expand bounded synonym and cross-reference previews
    #[arg(long)]
    pub full: bool,
}

#[derive(Args, Debug)]
pub struct VersionArgs {
    /// Include executable provenance and PATH diagnostics
    #[arg(long)]
    pub verbose: bool,
}

mod batch;
mod dispatch;
pub(crate) use self::batch::settle_batch;
pub(crate) use self::dispatch::{
    handle_batch, handle_cvx, handle_ddinter, handle_ema, handle_enrich, handle_gencc, handle_gtr,
    handle_uninstall, handle_version, handle_who, handle_who_ivd, version_identity_json,
};

#[cfg(test)]
mod tests;
