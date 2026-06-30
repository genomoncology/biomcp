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

## Protein and Phenotype Search JSON Metadata Seam

Protein and phenotype search JSON should opt into the metadata envelope instead
of using the bare generic search helper. This local dispatch seam check keeps the
routine gate deterministic without depending on public upstream availability.

```bash
cd ../.. && rg 'search_json_with_meta' src/cli/protein/dispatch.rs src/cli/phenotype/dispatch.rs | mustmatch like "src/cli/protein/dispatch.rs:
src/cli/phenotype/dispatch.rs:"
```
