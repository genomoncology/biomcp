//! Disease CLI payloads and subcommands.

use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct DiseaseSearchArgs {
    /// Free text query (disease name or keyword)
    #[arg(short, long)]
    pub query: Option<String>,
    /// Optional positional query alias for -q/--query
    #[arg(value_name = "QUERY")]
    pub positional_query: Option<String>,
    /// Restrict results by ontology source (mondo, doid, mesh)
    #[arg(long)]
    pub source: Option<String>,
    /// Inheritance: autosomal dominant/recessive, x-linked variants, y-linked, mitochondrial, multifactorial, oligogenic, polygenic, sporadic, somatic mosaicism, dominant/recessive, or an HPO inheritance ID
    #[arg(long)]
    pub inheritance: Option<String>,
    /// Filter by phenotype term (e.g., HP:0001250)
    #[arg(long)]
    pub phenotype: Option<String>,
    /// Onset: antenatal, embryonal, fetal, congenital, neonatal, infantile/infancy, childhood, juvenile, adolescent, young adult, adult, middle age, or late onset
    #[arg(long)]
    pub onset: Option<String>,
    /// Disable automatic discover fallback when zero direct disease rows are found
    #[arg(long)]
    pub no_fallback: bool,
    /// Maximum results, 1-50 (default: 10)
    #[arg(short, long, default_value = "10")]
    pub limit: usize,
    /// Skip the first N results
    #[arg(long, default_value = "0")]
    pub offset: usize,
}

#[derive(Args, Debug)]
pub struct DiseaseGetArgs {
    /// Explicit disease name or ID; use this for multi-word names before section tokens
    #[arg(long = "name", value_name = "NAME_OR_ID")]
    pub name_or_id: Option<String>,
    /// Disease name/ID followed by optional sections; with --name, all positional values are sections
    #[arg(value_name = "NAME_OR_SECTION", num_args = 0..)]
    pub args: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum DiseaseCommand {
    /// Search trials for this disease (best-effort)
    #[command(after_help = "\
EXAMPLES:
  biomcp disease trials melanoma --limit 5
  biomcp disease trials \"Rett Syndrome\" --limit 5
  biomcp disease trials \"lung cancer\" --source nci --limit 5

The supplied disease is sent as a literal condition. Results depend on source document wording.
See also: biomcp list disease")]
    Trials {
        /// Disease name (e.g., melanoma)
        name: String,
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
    /// Search articles for this disease (best-effort)
    #[command(after_help = "\
EXAMPLES:
  biomcp disease articles melanoma --limit 5
  biomcp disease articles \"glioblastoma\" --limit 5

Note: Searches free-text fields (e.g., eligibility criteria). Results depend on source document wording.
See also: biomcp list disease")]
    Articles {
        /// Disease name (e.g., melanoma)
        name: String,
        /// Maximum results, 1-50 (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Search drugs with this disease as an indication (best-effort)
    #[command(after_help = "\
EXAMPLES:
  biomcp disease drugs melanoma --limit 5
  biomcp disease drugs \"breast cancer\" --limit 5

Note: Searches free-text fields (e.g., eligibility criteria). Results depend on source document wording.
See also: biomcp list disease")]
    Drugs {
        /// Disease name (e.g., melanoma)
        name: String,
        /// Maximum results, 1-50 (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
}

mod dispatch;
pub(super) use self::dispatch::{handle_command, handle_get, handle_search};

#[cfg(test)]
mod tests;
