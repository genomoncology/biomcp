//! Gene CLI payloads and subcommands.

use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct GeneSearchArgs {
    /// Free text query (gene name, symbol, or keyword)
    #[arg(short, long)]
    pub query: Option<String>,
    /// Optional positional query alias for -q/--query
    #[arg(value_name = "QUERY")]
    pub positional_query: Option<String>,
    /// Filter by gene type (e.g., protein-coding, ncRNA, pseudo)
    #[arg(long = "type")]
    pub gene_type: Option<String>,
    /// Filter by chromosome (e.g., 7, X)
    #[arg(long)]
    pub chromosome: Option<String>,
    /// Filter by genomic region (chr:start-end)
    #[arg(long)]
    pub region: Option<String>,
    /// Filter by pathway ID/name (e.g., R-HSA-5673001)
    #[arg(long)]
    pub pathway: Option<String>,
    /// Filter by GO term ID/text (e.g., GO:0004672)
    #[arg(long = "go")]
    pub go_term: Option<String>,
    /// Maximum results, 1-50 (default: 10)
    #[arg(short, long, default_value = "10")]
    pub limit: usize,
    /// Skip the first N results
    #[arg(long, default_value = "0")]
    pub offset: usize,
}

#[derive(Args, Debug)]
pub struct GeneGetArgs {
    /// Gene symbol or known single-gene alias (e.g., BRAF, TP53, PD-L1)
    pub symbol: String,
    /// Sections to include (pathways, ontology, diseases, diagnostics, protein, go, interactions, civic, expression, hpa, druggability, clingen, gencc, constraint, disgenet, funding, all)
    #[arg(trailing_var_arg = true)]
    pub sections: Vec<String>,
}

#[derive(Args, Debug)]
pub struct CspecArgs {
    /// Stream an exact stored CSpec capture without refetching
    #[command(subcommand)]
    pub command: Option<CspecCommand>,
    /// HGNC gene symbol
    pub gene: Option<String>,
    /// Exact manifest resource IRI or unique short version
    #[arg(long, conflicts_with = "capture_id")]
    pub version: Option<String>,
    /// Page a previously selected CSpec capture without refetching
    #[arg(long, conflicts_with = "version")]
    pub capture_id: Option<String>,
    /// List metadata for linked public files without downloading them
    #[arg(long)]
    pub files: bool,
    /// Skip the first N criteria
    #[arg(long, default_value = "0")]
    pub offset: usize,
    /// Maximum criteria, 1-50 (default: 25)
    #[arg(long, default_value = "25")]
    pub limit: usize,
}

#[derive(Subcommand, Debug)]
pub enum CspecCommand {
    /// Stream an exact stored CSpec capture without refetching
    Document {
        /// Provider capture handle returned by CSpec selection
        capture_id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum GeneCommand {
    /// Show canonical gene definition card (same output as `get gene`)
    #[command(
        alias = "get",
        after_help = "\
EXAMPLES:
  biomcp gene definition BRAF
  biomcp gene get BRAF
  biomcp get gene BRAF

See also: biomcp list gene"
    )]
    Definition {
        /// HGNC gene symbol (e.g., BRAF)
        symbol: String,
    },
    /// Search trials linked to this gene symbol (best-effort)
    #[command(after_help = "\
EXAMPLES:
  biomcp gene trials BRAF --limit 5
  biomcp gene trials TP53 --limit 5
  biomcp gene trials EGFR --source nci --limit 5

The supplied symbol is sent as a literal biomarker. Results depend on source document wording.
See also: biomcp list gene")]
    Trials {
        /// HGNC gene symbol (e.g., BRAF)
        symbol: String,
        /// Maximum results, 1-50 (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
        /// Trial data source (ctgov or nci)
        #[arg(long, default_value = "ctgov")]
        source: String,
    },
    /// Search drugs targeting this gene symbol
    #[command(after_help = "\
EXAMPLES:
  biomcp gene drugs EGFR --limit 5
  biomcp gene drugs BRAF --limit 5

See also: biomcp list gene")]
    Drugs {
        /// HGNC gene symbol (e.g., BRAF)
        symbol: String,
        /// Maximum results, 1-50 (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Search articles mentioning this gene
    #[command(after_help = "\
EXAMPLES:
  biomcp gene articles BRAF --limit 5
  biomcp gene articles TP53 --limit 5

See also: biomcp list gene")]
    Articles {
        /// HGNC gene symbol (e.g., BRAF)
        symbol: String,
        /// Maximum results, 1-50 (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Retrieve versioned ClinGen Criteria Specification Registry source documents
    #[command(
        after_help = "Use an exact manifest resource IRI or unique short version with --version (for example, `--version 1.5.1`). Raw capture bytes are available only through `biomcp gene cspec document <capture-id>`."
    )]
    Cspec(CspecArgs),
    /// Show pathways section for this gene symbol
    #[command(after_help = "\
EXAMPLES:
  biomcp gene pathways BRAF
  biomcp gene pathways BRAF --limit 5 --offset 0
  biomcp gene pathways BRCA1

See also: biomcp list gene")]
    Pathways {
        /// HGNC gene symbol (e.g., BRAF)
        symbol: String,
        /// Maximum results, 1-25 (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    #[command(external_subcommand)]
    External(Vec<String>),
}

pub(super) mod cspec;
mod dispatch;
mod related;
#[cfg(test)]
pub(crate) use self::dispatch::render_loaded_card;
pub(crate) use self::dispatch::{handle_command, handle_get, handle_search};

#[cfg(test)]
#[path = "../../../tests/unit/cli/gene.rs"]
mod tests;
