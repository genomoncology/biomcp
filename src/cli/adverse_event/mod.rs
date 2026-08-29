//! Adverse-event CLI payloads.

use clap::Args;

#[derive(Args, Debug)]
pub struct AdverseEventSearchArgs {
    /// Drug name (required for FAERS queries)
    #[arg(short = 'd', long)]
    pub drug: Option<String>,
    /// Optional positional query alias for -d/--drug
    #[arg(value_name = "QUERY")]
    pub positional_query: Option<String>,
    /// Device name (required for --type device)
    #[arg(long)]
    pub device: Option<String>,
    /// Device manufacturer name (for --type device)
    #[arg(long)]
    pub manufacturer: Option<String>,
    /// Device product code (for --type device)
    #[arg(long = "product-code")]
    pub product_code: Option<String>,
    /// Filter by reaction term (MedDRA)
    #[arg(long)]
    pub reaction: Option<String>,
    /// Filter by reaction outcome [values: death, hospitalization, disability]
    #[arg(long)]
    pub outcome: Option<String>,
    /// Seriousness filter. FAERS: any, death, hospitalization, lifethreatening, disability, congenital, other. Device input any means death_or_injury; death and injury are exact
    #[arg(long, num_args = 0..=1, default_missing_value = "any")]
    pub serious: Option<String>,
    /// Received after year/date (YYYY or YYYY-MM-DD)
    #[arg(long = "date-from", alias = "since")]
    pub date_from: Option<String>,
    /// Received before year/date (YYYY or YYYY-MM-DD)
    #[arg(long = "date-to", alias = "until")]
    pub date_to: Option<String>,
    /// Restrict to suspect drugs only
    #[arg(long = "suspect-only")]
    pub suspect_only: bool,
    /// Patient sex filter (m|f)
    #[arg(long)]
    pub sex: Option<String>,
    /// Minimum patient age
    #[arg(long = "age-min")]
    pub age_min: Option<u32>,
    /// Maximum patient age
    #[arg(long = "age-max")]
    pub age_max: Option<u32>,
    /// Reporter qualification filter
    #[arg(long)]
    pub reporter: Option<String>,
    /// FAERS aggregation field: reaction/reactionmeddrapt, patient.reaction.reactionmeddrapt[.exact], patient.drug.medicinalproduct[.exact], patient.drug.openfda.generic_name[.exact], patient.drug.openfda.brand_name[.exact], patient.patientsex, patient.patientonsetage, serious, seriousnessdeath/hospitalization/lifethreatening/disabling/congenitalanomali/other, patient.reaction.reactionoutcome, or primarysource.qualification
    #[arg(long)]
    pub count: Option<String>,
    /// Query type: faers (default search path), recall, or device
    #[arg(long, default_value = "faers")]
    pub r#type: String,
    /// Adverse-event source [values: faers, vaers, all] (default: all; only applies to --type faers)
    #[arg(
        long,
        default_value = "all",
        value_name = "faers|vaers|all",
        value_parser = ["faers", "vaers", "all"]
    )]
    pub source: String,
    /// Filter by recall classification (Class I, Class II, Class III)
    #[arg(long)]
    pub classification: Option<String>,
    /// Maximum results, 1-50; also bounds VAERS top reactions (default: 10)
    #[arg(short, long, default_value = "10")]
    pub limit: usize,
    /// Skip the first N results
    #[arg(long, default_value = "0")]
    pub offset: usize,
}

#[derive(Args, Debug)]
pub struct AdverseEventGetArgs {
    /// FAERS safetyreportid or MAUDE mdr_report_key
    pub report_id: String,
    /// Sections to include (reactions, outcomes, concomitant, guidance, all)
    #[arg(trailing_var_arg = true)]
    pub sections: Vec<String>,
}

mod dispatch;
mod plan;
#[cfg(test)]
pub(crate) use self::dispatch::render_loaded_card;
pub(crate) use self::dispatch::{handle_get, handle_search};

#[cfg(test)]
mod tests;
