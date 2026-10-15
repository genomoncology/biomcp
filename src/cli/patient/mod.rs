//! Patient CLI payloads.

use clap::Args;

#[derive(Args, Debug)]
pub struct PatientGetArgs {
    /// FHIR Patient ID: 1-64 letters, digits, hyphens, or periods, not only periods
    pub id: String,
    /// Sections to include (conditions, all)
    #[arg(trailing_var_arg = true)]
    pub sections: Vec<String>,
}

/// Typed patient search: no free text, at least one filter.
#[derive(Args, Debug)]
pub struct PatientSearchArgs {
    /// Gender: male, female, other, or unknown
    #[arg(long)]
    pub gender: Option<String>,
    /// Birth date strictly after this FHIR date (YYYY, YYYY-MM, or YYYY-MM-DD)
    #[arg(long)]
    pub born_after: Option<String>,
    /// Birth date strictly before this FHIR date (YYYY, YYYY-MM, or YYYY-MM-DD)
    #[arg(long)]
    pub born_before: Option<String>,
    /// Coded condition as system|code (e.g. "http://snomed.info/sct|44054006")
    #[arg(long)]
    pub condition: Option<String>,
    /// Maximum results, 1-50 (default: 10)
    #[arg(
        short,
        long,
        default_value = "10",
        value_parser = clap::builder::RangedU64ValueParser::<usize>::new().range(1..=50)
    )]
    pub limit: usize,
    /// Print the count the server reports instead of the list
    #[arg(long)]
    pub count: bool,
}

mod dispatch;
pub(super) use self::dispatch::{handle_get, handle_search};

/// Help text for `biomcp get patient`, kept with the entity.
pub(super) const GET_AFTER_HELP: &str = "\
Reads one patient from the FHIR server named in BIOMCP_FHIR_BASE. The agent
never passes a URL. Patient commands run on the CLI and stdio MCP only;
`serve-http` refuses them.

EXAMPLES:
  BIOMCP_FHIR_BASE=https://fhir.example.org/fhir biomcp get patient <id>
  biomcp get patient <id> conditions

See also: biomcp list patient";
