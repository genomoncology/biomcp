//! The one transport check for patient records over MCP.
//!
//! `serve-http` has no authenticated per-user sessions, so it refuses every
//! command that reads a patient record. Every MCP tool reaches this check
//! through `BioMcpServer::execute_cli`.

use std::sync::OnceLock;

use crate::cli::{Cli, Commands, GetEntity, SearchEntity};

const HTTP_REFUSAL: &str = "Error: patient records are available on the CLI and stdio MCP only. `serve-http` refuses them until the HTTP transport has authenticated per-user sessions. See sdlc/issues/2026-09-11-health-record-entity-needs-authenticated-http-transport.md.";

/// Set once when this process starts `serve-http`. A process that serves HTTP
/// never serves stdio. One process-wide marker therefore covers every HTTP
/// path, including the modern-protocol dispatcher.
static HTTP_TRANSPORT: OnceLock<()> = OnceLock::new();

pub(in crate::mcp) fn mark_http_transport() {
    let _ = HTTP_TRANSPORT.set(());
}

/// Returns the refusal when this process serves HTTP and `cli` reads a patient.
pub(super) fn http_refusal(cli: &Cli) -> Option<&'static str> {
    (HTTP_TRANSPORT.get().is_some() && reads_patient_record(cli)).then_some(HTTP_REFUSAL)
}

/// True for `get patient`, `search patient`, and `batch patient`.
fn reads_patient_record(cli: &Cli) -> bool {
    match &cli.command {
        Commands::Get {
            entity: GetEntity::Patient(_),
        }
        | Commands::Search {
            entity: SearchEntity::Patient(_),
        } => true,
        Commands::Batch(args) => args.entity.trim().eq_ignore_ascii_case("patient"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::reads_patient_record;

    #[test]
    fn patient_record_commands_are_recognized_for_the_http_refusal() {
        let parse = |args: &[&str]| crate::cli::try_parse_cli(args.iter().copied()).expect("parse");
        for args in [
            &["biomcp", "get", "patient", "SYNTH-1"][..],
            &["biomcp", "get", "patient", "SYNTH-1", "conditions"],
            &["biomcp", "search", "patient"],
            &[
                "biomcp",
                "search",
                "patient",
                "--gender",
                "female",
                "--condition",
                "http://snomed.info/sct|44054006",
                "--count",
            ],
            &["biomcp", "batch", "patient", "SYNTH-1,SYNTH-2"],
            &["biomcp", "batch", " Patient ", "SYNTH-1"],
        ] {
            assert!(reads_patient_record(&parse(args)), "{args:?}");
        }
        for args in [
            &["biomcp", "get", "gene", "BRAF"][..],
            &["biomcp", "batch", "gene", "BRAF"],
            &["biomcp", "list", "patient"],
        ] {
            assert!(!reads_patient_record(&parse(args)), "{args:?}");
        }
    }

    #[test]
    fn typed_patient_search_takes_no_filters() {
        assert_eq!(
            super::super::search_args(super::super::TypedSearch(json!({"entity":"patient"})))
                .unwrap(),
            vec!["biomcp", "search", "patient"]
        );
        assert!(
            super::super::search_args(super::super::TypedSearch(
                json!({"entity":"patient","query":"x"})
            ))
            .is_err()
        );
        for field in [
            json!({"entity":"patient","limit":5}),
            json!({"entity":"patient","gender":"female"}),
            json!({"entity":"patient","condition":"http://snomed.info/sct|44054006"}),
            json!({"entity":"patient","count":true}),
        ] {
            assert!(
                super::super::search_args(super::super::TypedSearch(field.clone())).is_err(),
                "{field}"
            );
        }
    }
}
