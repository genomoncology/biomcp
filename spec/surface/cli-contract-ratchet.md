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

## Verify Lane Routes Source-Pending States

The live lane should route CPIC and NIH Reporter through explicit source
classification. Known auth or upstream-pending states should be reported to
operators, while unexpected response shapes remain product-red.

```bash
make -C ../.. -n verify 2>&1 | mustmatch like "tools/biomcp-verify-live cpic
tools/biomcp-verify-live nih-reporter"
```

## JSON Usage Errors Stay Parseable For Scripts

Scripts that opt into `--json` should be able to parse usage mistakes the same
way they parse command failures. A missing required argument is still invalid
usage and exits `2`, but stdout carries the standard JSON error envelope instead
of being empty.

<!-- mustmatch-lint: skip -->

```bash run id=json-usage-error exit=2
biomcp --json get variant
```

```json expect=json-usage-error contains
{
  "error": {
    "code": "invalid_argument"
  },
  "_meta": {
    "not_found": false
  }
}
```

```text expect=json-usage-error contains
"message":
```

## Skill Help Documents Worked-Example Selectors

The skill selector is intentionally positional: agents and humans can open a
worked example by number or slug without an extra `show` word. Help must keep
that runnable form visible so users do not stop at the generic `[COMMAND]`
placeholder.

```bash
biomcp skill --help | mustmatch like "EXAMPLES:
  biomcp skill 01
  biomcp skill article-follow-up"
```

## Parent Help Pages Give Concrete Next Commands

Parent and operator help pages should not be dead ends. Each high-value surface
shows at least one copy-pasteable next command before users need to open longer
documentation.

```bash
biomcp search --help | mustmatch like "EXAMPLES:
  biomcp search gene BRAF"
```

```bash
biomcp get --help | mustmatch like "EXAMPLES:
  biomcp get gene BRAF"
```

```bash
biomcp list --help | mustmatch like "EXAMPLES:
  biomcp list gene"
```

```bash
biomcp cache --help | mustmatch like "EXAMPLES:
  biomcp cache stats"
```

```bash
biomcp mcp --help | mustmatch like "EXAMPLES:
  biomcp mcp"
```

```bash
biomcp serve --help | mustmatch like "EXAMPLES:
  biomcp serve"
```

```bash
biomcp skill --help | mustmatch like "EXAMPLES:
  biomcp skill 01"
```

```bash
biomcp study --help | mustmatch like "EXAMPLES:
  biomcp study list"
```

## Retired Suggest Command Is Absent From Discovery

The command catalog should point agents at the living worked-example catalog,
not at the retired offline `suggest` router. Normal discovery surfaces must not
advertise `suggest` as a command or list page.

```bash
biomcp --help | mustmatch like "skill       BioMCP skill overview"
biomcp --help | mustmatch not '/(?m)^\s*suggest\s/'
```

```bash
biomcp list | mustmatch like '`skill list`'
biomcp list | mustmatch not '/`suggest\b/'
```

```bash
cd ../.. && uv run --no-sync python3 -c '
from pathlib import Path
paths = [Path("README.md"), *Path("docs").rglob("*.md"), *Path("skills").rglob("*.md")]
hits = []
for path in paths:
    text = path.read_text(encoding="utf-8")
    if "biomcp suggest" in text or "suggest <question>" in text:
        hits.append(str(path))
assert not hits, hits
print("shipped docs omit retired suggest command")
' | mustmatch like "shipped docs omit retired suggest command"
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

## Runtime next commands quote shell metacharacters

Runtime next commands are meant to be copied into a shell. When a local fallback
command carries a transcript-HGVS-shaped value with `>`, BioMCP should render the
argument as one quoted value instead of exposing a redirection operator.

```bash
biomcp --json discover 'NM_000248.3:c.1799T>A' | jq -r '._meta.next_commands[]?' | mustmatch like 'biomcp search article -k "NM_000248.3:c.1799T>A" --type review --limit 5'
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
