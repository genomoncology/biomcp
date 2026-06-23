//! Phenotype CLI payloads.

use clap::Args;

#[derive(Args, Debug)]
pub struct PhenotypeSearchArgs {
    /// HPO IDs (space- or comma-separated) or one symptom phrase / comma-separated symptom phrases
    pub terms: String,
    /// Maximum results, 1-50 (default: 10)
    #[arg(short, long, default_value = "10")]
    pub limit: usize,
    /// Skip the first N results
    #[arg(long, default_value = "0")]
    pub offset: usize,
}

mod dispatch;
pub(super) use self::dispatch::handle_search;

#[cfg(test)]
mod tests;
