//! Drug CLI payloads and subcommands.

use clap::{Args, Subcommand, ValueEnum};

use crate::cli::DrugRegionArg;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum WhoProductTypeArg {
    #[value(name = "finished_pharma")]
    FinishedPharma,
    #[value(name = "api")]
    Api,
    #[value(name = "vaccine")]
    Vaccine,
}

#[derive(Args, Debug)]
pub struct DrugSearchArgs {
    /// Free text query (drug name, class, etc.)
    #[arg(short, long)]
    pub query: Option<String>,
    /// Optional positional query alias for -q/--query
    #[arg(value_name = "QUERY")]
    pub positional_query: Option<String>,
    /// Filter by target gene symbol
    #[arg(long)]
    pub target: Option<String>,
    /// Filter by indication/disease name
    #[arg(long)]
    pub indication: Option<String>,
    /// Filter by mechanism text
    #[arg(long)]
    pub mechanism: Option<String>,
    /// Filter by drug type (e.g., biologic, small-molecule)
    #[arg(long = "type")]
    pub drug_type: Option<String>,
    /// Filter by ATC code
    #[arg(long)]
    pub atc: Option<String>,
    /// Filter by pharmacologic class
    #[arg(long = "pharm-class")]
    pub pharm_class: Option<String>,
    /// Filter by interaction partner drug name (currently unavailable from public data sources)
    #[arg(long)]
    pub interactions: Option<String>,
    /// Maximum results (default: 10)
    #[arg(short, long, default_value = "10")]
    pub limit: usize,
    /// Skip the first N results
    #[arg(long, default_value = "0")]
    pub offset: usize,
    /// Data region for drug regulatory context [default: all]
    #[arg(long, value_enum)]
    pub region: Option<DrugRegionArg>,
    /// WHO-only product type filter; requires explicit --region who
    #[arg(long = "product-type", value_enum, value_name = "PRODUCT_TYPE")]
    pub who_product_type: Option<WhoProductTypeArg>,
}

#[derive(Args, Debug)]
pub struct DrugGetArgs {
    /// Drug name (e.g., pembrolizumab, carboplatin)
    pub name: String,
    /// Sections to include (label, regulatory, safety, shortage, targets, indications, interactions, civic, approvals, all)
    pub sections: Vec<String>,
    /// Data region for regional sections (regulatory, safety, shortage, or all)
    #[arg(long, value_enum)]
    pub region: Option<DrugRegionArg>,
    /// Preserve raw FDA label subsections when used with `label` or `all`
    #[arg(long)]
    pub raw: bool,
}

#[derive(Subcommand, Debug)]
pub enum DrugCommand {
    /// Search trials using this drug (best-effort)
    #[command(after_help = "\
EXAMPLES:
  biomcp drug trials pembrolizumab --limit 5
  biomcp drug trials daraxonrasib --limit 20
  biomcp drug trials daraxonrasib --no-alias-expand --limit 20
  biomcp drug trials osimertinib --source nci --limit 5

Note: On `--source ctgov`, this helper inherits intervention alias expansion from `search trial`,
adds `Matched Intervention` / `matched_intervention_label` when an alternate alias matched first,
and supports `--no-alias-expand` for literal matching.
See also: biomcp list drug")]
    Trials {
        /// Drug name (e.g., pembrolizumab)
        name: String,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
        /// Trial data source (ctgov or nci)
        #[arg(long, default_value = "ctgov")]
        source: String,
        /// Disable ClinicalTrials.gov intervention alias expansion and force literal matching.
        #[arg(long = "no-alias-expand")]
        no_alias_expand: bool,
    },
    /// Search drug adverse events with FAERS-first lookup and CTGov fallback on FAERS 404
    #[command(after_help = "\
EXAMPLES:
  biomcp drug adverse-events pembrolizumab --limit 5
  biomcp drug adverse-events carboplatin --serious --limit 5

Note: FAERS 404 and FAERS 200-with-empty-results are reported differently. On FAERS 404,
this helper falls back to ClinicalTrials.gov trial-reported adverse events; JSON adds
faers_not_found and, when present, trial_adverse_events.
See also: biomcp list drug")]
    AdverseEvents {
        /// Drug name (e.g., pembrolizumab)
        name: String,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
        /// Serious reports only
        #[arg(long)]
        serious: bool,
    },
    /// Review DDInter-backed drug-drug interactions for one anchor drug
    #[command(after_help = "\
EXAMPLES:
  biomcp drug interactions warfarin
  biomcp drug interactions imatinib

Note: This helper reads the DDInter local download bundle from `BIOMCP_DDINTER_DIR`
or the default data directory. Use `biomcp ddinter sync` to force-refresh the
eight DDInter CSV files. Empty results are reported as
`no matching rows in the current DDInter download bundle` rather than as a claim
of no interactions.
See also: biomcp list drug")]
    Interactions {
        /// Drug name (e.g., warfarin)
        name: String,
    },
    #[command(external_subcommand)]
    External(Vec<String>),
}

mod dispatch;
pub(crate) use self::dispatch::{handle_command, handle_get, handle_search};

#[cfg(test)]
mod tests;
