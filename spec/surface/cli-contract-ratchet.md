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

## Disease Survival Commands Exit After Rendering

Disease survival cards are useful to agents only when the process exits after
printing them. These two command forms share the same disease-survival execution
path, so both are bounded by `timeout` and assert survival-card landmarks rather
than exact survival percentages.

```bash
set -o pipefail
cd ../..
timeout 20s ./tools/biomcp-ci get disease --name "chronic myeloid leukemia" survival | mustmatch like '## Survival (SEER Explorer)
Source: Chronic Myeloid Leukemia (CML)'
```

```bash
set -o pipefail
cd ../..
timeout 20s ./tools/biomcp-ci get disease "chronic myeloid leukemia" survival | mustmatch like '## Survival (SEER Explorer)
Source: Chronic Myeloid Leukemia (CML)'
```

## Cache Max-Age Env Override Is Reflected in Cache Stats

The cache configuration reference promises an operator env override for the
managed HTTP cache age limit. The local cache stats command exposes the resolved
limit and its origin, so operators can confirm that an env override took effect
without touching public upstream services.

```bash
BIOMCP_CACHE_MAX_AGE=172800 biomcp --json cache stats | mustmatch like '"max_age_secs": 172800
"max_age_origin": "env"'
```

## Protein and Phenotype Search JSON Metadata Seam

Protein and phenotype search JSON should opt into the metadata envelope instead
of using the bare generic search helper. These fixture-backed tests exercise the
local JSON envelope and parse every emitted follow-up command, so the routine
ratchet does not depend on public upstream availability.

```bash
cd ../..
cargo test --locked protein_search_json_next_commands_parse --lib >/tmp/biomcp-protein-json-next-commands.log
cargo test --locked phenotype_search_json_next_commands_parse --lib >/tmp/biomcp-phenotype-json-next-commands.log
printf 'protein and phenotype JSON next-command fixture tests passed\n' | mustmatch like "fixture tests passed"
```
