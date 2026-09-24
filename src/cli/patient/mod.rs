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

/// `search patient` filters. It takes no free text.
#[derive(Args, Debug)]
pub struct PatientSearchArgs {
    /// Administrative gender: male, female, other, or unknown
    #[arg(long)]
    pub gender: Option<String>,
    /// Born strictly after this FHIR date: YYYY, YYYY-MM, or YYYY-MM-DD
    #[arg(long, value_name = "DATE")]
    pub born_after: Option<String>,
    /// Born strictly before this FHIR date: YYYY, YYYY-MM, or YYYY-MM-DD
    #[arg(long, value_name = "DATE")]
    pub born_before: Option<String>,
    /// Has a Condition with this code, as system|code
    #[arg(long, value_name = "SYSTEM|CODE")]
    pub condition: Option<String>,
    /// Maximum patients, 1-50 (default: 10); ignored with --count
    #[arg(short, long, default_value = "10")]
    pub limit: usize,
    /// Print the total the server reports instead of patients
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

/// Help text for `biomcp search patient`, kept with the entity.
pub(super) const SEARCH_AFTER_HELP: &str = "\
Finds patients on the FHIR server named in BIOMCP_FHIR_BASE by gender, birth
date range, and one coded condition. It takes no free text and needs at least
one filter. BioMCP first reads the server's metadata and refuses a filter the
server does not list. The search asks for strict handling and reads one page.
Output keeps only each patient's id, gender, and birth date.

EXAMPLES:
  biomcp search patient --gender female --born-after 1950-01-01 --condition \"http://snomed.info/sct|44054006\" --limit 10
  biomcp search patient --condition \"http://snomed.info/sct|44054006\" --count

See also: biomcp list patient";
