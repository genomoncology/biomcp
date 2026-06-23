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
    /// Gene symbol (e.g., BRAF, TP53, EGFR)
    pub symbol: String,
    /// Sections to include (pathways, ontology, diseases, diagnostics, protein, go, interactions, civic, expression, hpa, druggability, clingen, constraint, disgenet, funding, all)
    #[arg(trailing_var_arg = true)]
    pub sections: Vec<String>,
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
  biomcp gene trials SHANK3 --limit 5
  biomcp gene trials EGFR --source nci --limit 5

Note: CTGov uses a bounded rare-disease condition plan for supported genes such as SHANK3; other genes and sources keep the literal gene search. Results depend on source document wording.
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

mod dispatch;
mod related;
pub(crate) use self::dispatch::{handle_command, handle_get, handle_search};

#[cfg(test)]
mod tests;
