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
    /// Testing recommendation: Actionable PGx, Informative PGx, No Clinical PGx, Testing Recommended, or Testing Required
    #[arg(long = "pgx-testing")]
    pub pgx_testing: Option<String>,
    /// Best-effort match over guideline names or CPIC levels
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
    /// Sections to include (interactions, recommendations, frequencies, guidelines, annotations, all)
    #[arg(value_name = "SECTION")]
    pub sections: Vec<String>,
    /// Maximum rows per requested section, 1-50 (default: 10)
    #[arg(short, long, default_value = "10")]
    pub limit: usize,
    /// Skip the first N rows (one requested section only)
    #[arg(long, default_value = "0")]
    pub offset: usize,
    /// Include every section with a bounded limit of 50 per section
    #[arg(long)]
    pub full: bool,
}

mod dispatch;
#[cfg(test)]
pub(crate) use self::dispatch::render_loaded_card;
pub(super) use self::dispatch::{handle_get, handle_search};

#[cfg(test)]
mod tests;
