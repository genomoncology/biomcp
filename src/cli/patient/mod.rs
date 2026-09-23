//! Patient CLI payloads.

use clap::Args;

#[derive(Args, Debug)]
pub struct PatientGetArgs {
    /// FHIR Patient ID: 1-64 letters, digits, hyphens, or periods
    pub id: String,
    /// Sections to include (conditions, all)
    #[arg(trailing_var_arg = true)]
    pub sections: Vec<String>,
}

/// `search patient` takes no filters until patient search ships.
#[derive(Args, Debug)]
pub struct PatientSearchArgs {}

mod dispatch;
pub(super) use self::dispatch::{handle_get, handle_search};

/// The refusal `search patient` returns until patient search ships.
pub(crate) const SEARCH_NOT_YET_AVAILABLE: &str =
    "search patient is not yet available. Use `biomcp get patient <id>` for one known patient.";

/// Help text for `biomcp get patient`, kept with the entity.
pub(super) const GET_AFTER_HELP: &str = "\
Reads one patient from the FHIR server named in BIOMCP_FHIR_BASE. The agent
never passes a URL. Patient commands run on the CLI and stdio MCP only;
`serve-http` refuses them.

EXAMPLES:
  BIOMCP_FHIR_BASE=https://fhir.example.org/fhir biomcp get patient <id>
  biomcp get patient <id> conditions

See also: biomcp list patient";
