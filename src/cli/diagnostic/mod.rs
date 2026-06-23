//! Diagnostic CLI payloads.

use clap::{Args, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DiagnosticSourceArg {
    Gtr,
    #[value(name = "who-ivd")]
    WhoIvd,
    All,
}

impl From<DiagnosticSourceArg> for crate::entities::diagnostic::DiagnosticSourceFilter {
    fn from(value: DiagnosticSourceArg) -> Self {
        match value {
            DiagnosticSourceArg::Gtr => Self::Gtr,
            DiagnosticSourceArg::WhoIvd => Self::WhoIvd,
            DiagnosticSourceArg::All => Self::All,
        }
    }
}

#[derive(Args, Debug)]
pub struct DiagnosticSearchArgs {
    /// Diagnostic source [default: all]
    #[arg(long, value_enum, default_value_t = DiagnosticSourceArg::All)]
    pub source: DiagnosticSourceArg,
    /// Filter by gene symbol
    #[arg(short = 'g', long)]
    pub gene: Option<String>,
    /// Filter by disease phrase (min 3 alphanumeric chars, word-boundary match)
    #[arg(short = 'd', long)]
    pub disease: Option<String>,
    /// Filter by exact diagnostic type
    #[arg(short = 't', long = "type")]
    pub test_type: Option<String>,
    /// Filter by manufacturer or lab substring
    #[arg(long)]
    pub manufacturer: Option<String>,
    /// Maximum results, 1-50 (default: 10)
    #[arg(short, long, default_value = "10")]
    pub limit: usize,
    /// Skip the first N results
    #[arg(long, default_value = "0")]
    pub offset: usize,
}

#[derive(Args, Debug)]
pub struct DiagnosticGetArgs {
    /// Diagnostic accession or WHO IVD product code (e.g., GTR000006692.3 or "ITPW02232- TC40")
    pub accession: String,
    /// Sections to include (genes, conditions, methods, regulatory, all)
    #[arg(trailing_var_arg = true)]
    pub sections: Vec<String>,
}

mod dispatch;
pub(super) use self::dispatch::{handle_get, handle_search};

#[cfg(test)]
mod tests;
