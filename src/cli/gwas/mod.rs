//! GWAS CLI payloads.

use clap::Args;

#[derive(Args, Debug)]
pub struct GwasSearchArgs {
    /// Filter by gene symbol
    #[arg(short = 'g', long)]
    pub gene: Option<String>,
    /// Optional positional query alias for -g/--gene
    #[arg(value_name = "QUERY")]
    pub positional_query: Option<String>,
    /// Filter by disease trait text
    #[arg(long = "trait")]
    pub trait_query: Option<String>,
    /// Filter by genomic region (chr:start-end)
    #[arg(long)]
    pub region: Option<String>,
    /// Filter by p-value threshold
    #[arg(long = "p-value")]
    pub p_value: Option<f64>,
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
