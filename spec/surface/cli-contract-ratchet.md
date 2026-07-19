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

## GWAS p-values are probabilities

<!-- mustmatch-lint: skip -->

A GWAS p-value threshold narrows associations only when it is a finite
probability greater than zero and at most one. Invalid thresholds fail locally
instead of returning confident emptiness or silently disabling the filter.

| str:value | str:label |
|---|---|
| NaN | not a number |
| +inf | positive infinity |
| -inf | negative infinity |
| 1e309 | overflow |
| 0 | zero |
| -0.01 | negative |
| 1.01 | greater than one |

```bash run id=invalid-gwas-p-value exit=2 each_row="GWAS p-values are probabilities"
biomcp --json search gwas BRAF --p-value={{value}} --limit 1
```

```json expect=invalid-gwas-p-value contains each_row="GWAS p-values are probabilities"
{
  "error": {
    "code": "invalid_argument"
  }
}
```

## Trial ages stay within a human range

<!-- mustmatch-lint: skip -->

Trial eligibility accepts fractional patient ages from zero through 150 years.
Non-finite or out-of-range ages fail before count or search work, so they cannot
silently change a result page.

| str:value | str:label |
|---|---|
| NaN | not a number |
| +inf | positive infinity |
| -inf | negative infinity |
| 1e309 | overflow |
| -1 | below zero |
| 151 | above 150 |

```bash run id=invalid-trial-age exit=2 each_row="Trial ages stay within a human range"
biomcp --json search trial --age={{value}} --count-only
```

```json expect=invalid-trial-age contains each_row="Trial ages stay within a human range"
{
  "error": {
    "code": "invalid_argument"
  }
}
```

## Trial coordinates stay on Earth

<!-- mustmatch-lint: skip -->

Geographic trial search uses finite latitude from -90 through 90 and longitude
from -180 through 180. Values outside those bounds are client errors, not
ClinicalTrials.gov or NCI outages.

| str:coordinates | str:label |
|---|---|
| --lat=NaN --lon=0 | latitude not a number |
| --lat=+inf --lon=0 | latitude infinity |
| --lat=-91 --lon=0 | latitude below -90 |
| --lat=91 --lon=0 | latitude above 90 |
| --lat=0 --lon=NaN | longitude not a number |
| --lat=0 --lon=+inf | longitude infinity |
| --lat=0 --lon=-181 | longitude below -180 |
| --lat=0 --lon=181 | longitude above 180 |

```bash run id=invalid-trial-coordinate exit=2 each_row="Trial coordinates stay on Earth"
biomcp --json search trial -c melanoma {{coordinates}} --distance=1 --limit=1
```

```json expect=invalid-trial-coordinate contains each_row="Trial coordinates stay on Earth"
{
  "error": {
    "code": "invalid_argument"
  }
}
```

## Discover rejects oversized input locally

<!-- mustmatch-lint: skip -->

Discover accepts a trimmed free-text query up to 4,096 UTF-8 bytes. Longer text
fails as an invalid argument before ontology clients are constructed, without
exposing an internal backend name.

```bash run id=oversized-discover exit=2
bash ../fixtures/run-oversized-discover-query.sh
```

```json expect=oversized-discover contains
{
  "error": {
    "code": "invalid_argument"
  }
}
```

```text expect=oversized-discover not-contains
OLS4
ols4
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

## Trial mutation help explains inclusion verification

A mutation-bearing caller should learn at the flag itself that BioMCP checks
simple molecular text against registry eligibility after broad CTGov discovery.
The help also keeps the recall and boolean-expression boundaries explicit.

```bash
biomcp search trial --help | mustmatch like "simple mutation
registry eligibility
exclusion-only
absent
boolean
discovery-only"
```

## Trial mutation list reference explains inclusion verification

The agent-facing list page is the compact trial reference, so it must teach the
same inclusion/exclusion behavior and limits rather than describing only the
upstream free-text query.

```bash
biomcp list trial | mustmatch like "simple mutation
registry eligibility
exclusion-only
absent
boolean
discovery-only"
```

## Root trial reference explains inclusion verification

The root list is a checked command reference as well as an index. Its trial
filter summary should not preserve the old broad-discovery description after
the more detailed trial page changes.

```bash
biomcp list | mustmatch like "simple mutation
registry eligibility
exclusion-only
absent
boolean
discovery-only"
```

## Trial guide explains mutation inclusion verification

The trial guide should describe the precision-oriented eligibility check without
turning absent registry wording or boolean expressions into strict matches.

```bash
cat ../../docs/user-guide/trial.md | mustmatch like "simple mutation
registry eligibility
exclusion-only
absent
boolean
discovery-only"
```

## CLI reference explains mutation inclusion verification

The long CLI reference should stay aligned with help, both list surfaces, and
the focused trial guide.

```bash
cat ../../docs/user-guide/cli-reference.md | mustmatch like "simple mutation
registry eligibility
exclusion-only
absent
boolean
discovery-only"
```

## UX architecture reference explains mutation inclusion verification

The maintained UX architecture reference should match the shipped CLI guide rather
than preserving the old broad-discovery-only behavior.

```bash
cat ../../architecture/ux/cli-reference.md | mustmatch like "simple mutation
registry eligibility
exclusion-only
absent
boolean
discovery-only"
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
<!-- mustmatch-lint: skip -->

Runtime next commands are meant to be copied into a shell. When a local fallback
command carries a transcript-HGVS-shaped value with `>`, BioMCP should render the
argument as one quoted value instead of exposing a redirection operator.

```bash
cargo test --lib entities::discover::tests::empty_discover_result_quotes_shell_metacharacters_in_json_next_command -- --exact
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
