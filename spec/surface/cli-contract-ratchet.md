# CLI Contract Ratchet

BioMCP keeps the public command surface in several places: clap help, `biomcp
list`, user documentation, architecture references, executable specs, and JSON
metadata. This local contract keeps the routine gate wired to the whole-surface
ratchet so drift cannot hide until review.

## Whole-Surface Contract Ratchet Runs in the Routine Gate

The whole-surface ratchet is a local contract: it checks the shipped command
surface and the source-controlled exception registry, not public upstream data.
The pytest contract below is the durable entrypoint for that policy.

```bash
set -o pipefail
cd ../.. && uv run --no-sync pytest tests/test_cli_surface_contract_ratchet.py -v 2>&1 | mustmatch like "test_quality_ratchet_runs_whole_surface_cli_contract
test_cli_surface_contract_exception_registry_names_initial_exceptions"
```

## Entity Search JSON Next-Command Matrix

Entity search JSON should teach the next executable command unless a surface is
explicitly listed in the exception registry. The local Rust matrix uses fixture
rows, so this routine check proves the envelope contract without depending on
public upstream availability.

```bash
set -o pipefail
cd ../.. && cargo test --locked cli::tests::next_commands_json_property::search_surfaces::search_entity_json_next_commands_matrix_covers_protein_and_phenotype -- --exact 2>&1 | mustmatch like "search_entity_json_next_commands_matrix_covers_protein_and_phenotype"
```
