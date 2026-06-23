//! Pharmacogenomics CLI payloads.

use clap::Args;

#[derive(Args, Debug)]
pub struct PgxSearchArgs {
    /// Filter by gene symbol
    #[arg(short = 'g', long)]
    pub gene: Option<String>,
    /// Optional positional query alias for -g/--gene
    #[arg(value_name = "QUERY")]
    pub positional_query: Option<String>,
    /// Filter by drug name
    #[arg(short = 'd', long)]
    pub drug: Option<String>,
    /// Filter by CPIC level (A/B/C/D)
    #[arg(long = "cpic-level")]
    pub cpic_level: Option<String>,
    /// Filter by PGx testing recommendation
    #[arg(long = "pgx-testing")]
    pub pgx_testing: Option<String>,
    /// Filter by evidence level (best-effort)
    #[arg(long)]
    pub evidence: Option<String>,
    /// Maximum results, 1-50 (default: 10)
    #[arg(short, long, default_value = "10")]
    pub limit: usize,
    /// Skip the first N results
    #[arg(long, default_value = "0")]
    pub offset: usize,
}

#[derive(Args, Debug)]
pub struct PgxGetArgs {
    /// Gene symbol or drug name (e.g., CYP2D6, codeine)
    pub query: String,
    /// Sections to include (recommendations, frequencies, guidelines, annotations, all)
    #[arg(trailing_var_arg = true)]
    pub sections: Vec<String>,
}

mod dispatch;
pub(super) use self::dispatch::{handle_get, handle_search};

#[cfg(test)]
mod tests;
