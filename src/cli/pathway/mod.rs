//! Pathway CLI payloads and subcommands.

use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct PathwaySearchArgs {
    /// Free text query (pathway name, process, keyword)
    #[arg(short, long)]
    pub query: Option<String>,
    /// Positional alias for -q/--query; required unless --top-level is present, and multi-word queries must be quoted
    #[arg(value_name = "QUERY")]
    pub positional_query: Option<String>,
    /// Entity type filter (e.g., pathway)
    #[arg(long = "type")]
    pub pathway_type: Option<String>,
    /// Include top-level pathways
    #[arg(long = "top-level")]
    pub top_level: bool,
    /// Maximum results, 1-25 (default: 10)
    #[arg(short, long, default_value = "10")]
    pub limit: usize,
    /// Skip the first N results
    #[arg(long, default_value = "0")]
    pub offset: usize,
}

#[derive(Args, Debug)]
pub struct PathwayGetArgs {
    /// Pathway ID (e.g., R-HSA-5673001, hsa05200)
    pub id: String,
    /// Sections to include (genes, events (Reactome only), enrichment (Reactome only), all = all sections available for the resolved source)
    #[arg(trailing_var_arg = true)]
    pub sections: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum PathwayCommand {
    /// Search drugs linked to genes in this pathway (best-effort)
    #[command(after_help = "\
EXAMPLES:
  biomcp pathway drugs R-HSA-5673001 --limit 5
  biomcp pathway drugs hsa05200 --limit 5
  biomcp pathway drugs R-HSA-6802957 --limit 5

Note: Searches free-text fields (e.g., eligibility criteria). Results depend on source document wording.
See also: biomcp list pathway")]
    Drugs {
        /// Pathway ID (e.g., R-HSA-5673001, hsa05200)
        id: String,
        /// Maximum results, 1-50 (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Search articles linked to this pathway (best-effort)
    #[command(after_help = "\
EXAMPLES:
  biomcp pathway articles R-HSA-5673001 --limit 5
  biomcp pathway articles hsa05200 --limit 5
  biomcp pathway articles R-HSA-6802957 --limit 5

Note: Searches free-text fields (e.g., eligibility criteria). Results depend on source document wording.
See also: biomcp list pathway")]
    Articles {
        /// Pathway ID (e.g., R-HSA-5673001, hsa05200)
        id: String,
        /// Maximum results, 1-50 (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Search trials linked to this pathway (best-effort)
    #[command(after_help = "\
EXAMPLES:
  biomcp pathway trials R-HSA-5673001 --limit 5
  biomcp pathway trials hsa05200 --limit 5
  biomcp pathway trials R-HSA-5673001 --source nci --limit 5

Note: Searches free-text fields (e.g., eligibility criteria). Results depend on source document wording.
See also: biomcp list pathway")]
    Trials {
        /// Pathway ID (e.g., R-HSA-5673001, hsa05200)
        id: String,
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
}

mod dispatch;
pub(super) use self::dispatch::{handle_command, handle_get, handle_search};

#[cfg(test)]
mod tests;
