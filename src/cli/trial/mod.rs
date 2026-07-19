//! Trial CLI payloads.

use clap::Args;

#[derive(Args, Debug)]
pub struct TrialSearchArgs {
    /// Filter by condition/disease
    #[arg(short = 'c', long, num_args = 1..)]
    pub condition: Vec<String>,
    /// Optional positional query alias for -c/--condition
    #[arg(value_name = "QUERY")]
    pub positional_query: Option<String>,
    /// Filter by intervention/drug.
    ///
    /// On `--source ctgov`, BioMCP sends each intervention as one quoted
    /// literal and expands only plausible trade names and investigational
    /// codes. A rejected expanded alias preserves successful requested-name
    /// results but makes the exact total unknown. Use `--no-alias-expand` for
    /// one literal request.
    #[arg(short = 'i', long, num_args = 1..)]
    pub intervention: Vec<String>,
    /// Disable ClinicalTrials.gov intervention alias expansion and send one literal request.
    #[arg(long = "no-alias-expand")]
    pub no_alias_expand: bool,
    /// Filter by institution/facility name (text-search mode by default).
    ///
    /// Without `--lat`/`--lon`/`--distance`, this uses cheap CTGov
    /// `query.locn` text-search mode. With all three geo flags, it enters
    /// geo-verify mode and performs extra per-study location fetches to
    /// confirm the facility match within the requested distance. Geo-verify
    /// mode is materially more expensive, especially with `--count-only`.
    #[arg(long, num_args = 1..)]
    pub facility: Vec<String>,
    /// Filter by phase. Canonical CLI forms: NA, 1, 1/2, 2, 3, 4.
    /// Accepted aliases: EARLY_PHASE1, PHASE1, PHASE2, PHASE3, PHASE4.
    ///
    /// `1/2` matches the ClinicalTrials.gov combined Phase 1/Phase 2 label
    /// (studies tagged as both phases), not Phase 1 OR Phase 2.
    #[arg(short = 'p', long)]
    pub phase: Option<String>,
    /// Study type (e.g., interventional, observational)
    #[arg(long = "study-type")]
    pub study_type: Option<String>,
    /// Finite patient age from 0 through 150 years (decimals accepted, e.g. 0.5 for 6 months).
    ///
    /// With `--count-only`, age-only CTGov searches report an approximate
    /// upstream total because BioMCP applies the age filter during full
    /// search, not the fast count path.
    #[arg(long)]
    pub age: Option<f32>,
    /// Eligible sex filter [values: female, male, all].
    ///
    /// `all` (also `any`/`both`) resolves to no sex restriction, so no sex
    /// filter is sent to ClinicalTrials.gov. Use `female` or `male` to
    /// apply an actual restriction.
    #[arg(long)]
    pub sex: Option<String>,
    /// Filter by trial status [values: recruiting, not_yet_recruiting, enrolling_by_invitation, active_not_recruiting, completed, suspended, terminated, withdrawn]
    #[arg(short = 's', long)]
    pub status: Option<String>,
    /// CTGov exact free-text boolean over title, summary, eligibility, and keywords.
    ///
    /// After broad discovery, simple mutation text is checked against registry
    /// eligibility to remove exclusion-only matches; trials where the term is
    /// absent remain discoverable, while boolean expressions are discovery-only.
    #[arg(long, num_args = 1..)]
    pub mutation: Vec<String>,
    /// Search eligibility criteria with free-text terms (best-effort)
    #[arg(long, num_args = 1..)]
    pub criteria: Vec<String>,
    /// Biomarker phrase search: NCI CTS native; CTGov keyword/intervention/condition. Prefer for gene-level trial broadening.
    #[arg(long, num_args = 1..)]
    pub biomarker: Vec<String>,
    /// Prior therapy mentioned in eligibility
    #[arg(long, num_args = 1..)]
    pub prior_therapies: Vec<String>,
    /// Drug/therapy patient progressed on
    #[arg(long, num_args = 1..)]
    pub progression_on: Vec<String>,
    /// Line of therapy: 1L, 2L, 3L+
    #[arg(long)]
    pub line_of_therapy: Option<String>,
    /// Filter by sponsor (best-effort)
    #[arg(long, num_args = 1..)]
    pub sponsor: Vec<String>,
    /// Sponsor/funder category [values: nih, industry, fed, other]
    #[arg(long = "sponsor-type")]
    pub sponsor_type: Option<String>,
    /// Trials updated after date (YYYY-MM-DD)
    #[arg(long = "date-from", alias = "since")]
    pub date_from: Option<String>,
    /// Trials updated before date (YYYY-MM-DD)
    #[arg(long = "date-to", alias = "until")]
    pub date_to: Option<String>,
    /// Finite latitude from -90 through 90 for geographic search
    #[arg(long, allow_hyphen_values = true)]
    pub lat: Option<f64>,
    /// Finite longitude from -180 through 180 for geographic search
    #[arg(long, allow_hyphen_values = true)]
    pub lon: Option<f64>,
    /// Distance (miles) for geographic search
    #[arg(long)]
    pub distance: Option<u32>,
    /// Only return trials with posted results (default: off, include trials with/without posted results)
    #[arg(long = "has-results", visible_alias = "results-available")]
    pub results_available: bool,
    /// Return only total count (no result table).
    ///
    /// Alias-expanded `--intervention` counts may take longer and can return
    /// `unknown` when the merged traversal cap is reached.
    #[arg(long = "count-only")]
    pub count_only: bool,
    /// Trial data source (ctgov or nci)
    #[arg(long, default_value = "ctgov")]
    pub source: String,
    /// Skip the first N results (pagination)
    #[arg(long, default_value = "0")]
    pub offset: usize,
    /// Cursor token from a previous response.
    ///
    /// When CTGov intervention alias expansion fans out to multiple queries,
    /// `--next-page` is unavailable; use `--offset` or `--no-alias-expand`.
    #[arg(long = "next-page")]
    pub next_page: Option<String>,
    /// Maximum results, 1-50 (default: 10)
    #[arg(short, long, default_value = "10")]
    pub limit: usize,
}

#[derive(Args, Debug)]
pub struct TrialGetArgs {
    /// ClinicalTrials.gov identifier (e.g., NCT02693535)
    pub nct_id: String,
    /// Sections or document form (eligibility, contacts, locations, outcomes, arms, references, all, documents, document <filename>)
    #[arg(trailing_var_arg = true)]
    pub sections: Vec<String>,
    /// Trial data source (ctgov or nci)
    #[arg(long, default_value = "ctgov")]
    pub source: String,
    /// Skip the first N locations (requires locations section)
    #[arg(long)]
    pub offset: Option<usize>,
    /// Maximum locations to show (requires locations section; must be >= 1)
    #[arg(long)]
    pub limit: Option<usize>,
}

mod dispatch;
mod documents;
mod zero_result;
pub(super) use self::dispatch::{handle_get, handle_search};

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_locations;
