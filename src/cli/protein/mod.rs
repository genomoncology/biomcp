//! Protein CLI payloads and subcommands.

use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct ProteinSearchArgs {
    /// Free text query (protein name, accession, keyword)
    #[arg(short, long)]
    pub query: Option<String>,
    /// Optional positional query alias for -q/--query
    #[arg(value_name = "QUERY")]
    pub positional_query: Option<String>,
    /// Include all species (reviewed entries remain the default)
    #[arg(long)]
    pub all_species: bool,
    /// Explicitly restrict to reviewed entries (the default)
    #[arg(long, conflicts_with = "include_unreviewed")]
    pub reviewed: bool,
    /// Include unreviewed entries without changing the species scope
    #[arg(long)]
    pub include_unreviewed: bool,
    /// Filter by disease text
    #[arg(long)]
    pub disease: Option<String>,
    /// Filter by protein existence level (1-5)
    #[arg(long)]
    pub existence: Option<u8>,
    /// Maximum results, 1-100 (default: 10)
    #[arg(short, long, default_value = "10")]
    pub limit: usize,
    /// Skip the first N results
    #[arg(long, default_value = "0")]
    pub offset: usize,
    /// Cursor token from a previous response
    #[arg(long = "next-page")]
    pub next_page: Option<String>,
}

#[derive(Args, Debug)]
pub struct ProteinGetArgs {
    /// UniProt accession or HGNC symbol (e.g., P15056 or BRAF)
    pub accession: String,
    /// Sections to include (domains, interactions, complexes, structures, all)
    #[arg(trailing_var_arg = true)]
    pub sections: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum ProteinCommand {
    /// Show protein structural identifiers
    #[command(after_help = "\
EXAMPLES:
  biomcp protein structures P15056
  biomcp protein structures P15056 --limit 25 --offset 5

See also: biomcp list protein")]
    Structures {
        /// UniProt accession or HGNC symbol (e.g., P15056 or BRAF)
        accession: String,
        /// Maximum structures to show, 1-100 (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
}

mod dispatch;
#[cfg(test)]
pub(crate) use self::dispatch::render_loaded_card;
pub(super) use self::dispatch::{handle_command, handle_get, handle_search};

#[cfg(test)]
mod tests;
