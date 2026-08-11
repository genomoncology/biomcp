# CLI Contract Ratchet

BioMCP keeps the public command surface in several places: clap help, `biomcp
list`, user documentation, architecture references, executable specs, and JSON
metadata. This local contract keeps the routine gate wired to the whole-surface
ratchet so drift cannot hide until review.

## Whole-Surface Contract Ratchet Is Owned by the Test Gate

The whole-surface ratchet is a local contract: it checks the shipped command
surface and the source-controlled exception registry, not public upstream data.
`tests/test_cli_surface_contract_ratchet.py` is the durable entrypoint and runs
under `make test`. This executable-product spec does not recursively launch the
source-policy test gate.

## Verify Lane Routes Source-Pending States

The live lane should route NIH Reporter through explicit source classification.
Known upstream-pending states should be reported to operators, while unexpected
response shapes remain product-red. CPIC PGx proof is receipt-backed and routine.

```bash
make -C ../.. -n verify 2>&1 | mustmatch like "tools/biomcp-verify-live nih-reporter"
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

## Free-string filters reject unknown values locally

<!-- mustmatch-lint: skip -->

Disease inheritance and onset, adverse-event aggregation fields, and PGx testing
recommendations each have a supported vocabulary. Unknown values fail as invalid
arguments instead of returning a successful empty page or bucket list.

| str:command | str:label |
|---|---|
| biomcp --json search disease -q melanoma --inheritance zzqqxx_not_a_pattern --limit 1 --no-fallback | unknown inheritance |
| biomcp --json search disease -q melanoma --onset zzqqxx_not_an_onset --limit 1 --no-fallback | unknown onset |
| biomcp --json search adverse-event -d aspirin --count bogusfield --limit 1 | unknown count field |
| biomcp --json search pgx -g CYP2D6 --pgx-testing bogusrec --limit 1 | unknown PGx testing value |

```bash run id=invalid-free-string-filter exit=2 each_row="Free-string filters reject unknown values locally"
{{command}}
```

```json expect=invalid-free-string-filter contains each_row="Free-string filters reject unknown values locally"
{
  "error": {
    "code": "invalid_argument"
  }
}
```

## Cache stats separate provider-capture storage from HTTP cache storage

Provider captures are a bounded internal store, not HTTP CACache blobs. Operators
can see their aggregate retained size without exposing a capture path or response
content; the existing HTTP blob metrics retain their original meaning.

```bash
biomcp --json cache stats | mustmatch like '"provider_capture_bytes":'
```

```bash
biomcp --json cache clean --dry-run | mustmatch like '"provider_capture_bytes_freed":'
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

## ClinGen Opt-In Routes Remain Distinct From Ordinary Defaults

ClinGen's ERepo, CSpec, Allele Registry, and linked-data identity evidence are
separate opt-in capabilities even though they share an organization name. Their
local discovery surfaces remain available together without turning verification
into the ordinary variant-article path.

```bash
biomcp variant --help | mustmatch like 'erepo
Retrieve versioned ClinGen ERepo expert assertions by CAid'
biomcp gene --help | mustmatch like 'cspec
ClinGen Criteria Specification Registry source documents'
biomcp variant normalize --help | mustmatch like 'CAR is available as car
biomcp variant normalize car'
biomcp variant articles --help | mustmatch like '--verify-identity
[default: union]'
```

## Converted ClinGen Contracts Leave the Live Lane

ClinGen CAR and LDH response contracts are replayed from receipted local captures.
Their former provider-health pages must therefore leave the opt-in live lane;
routine proof owns the deterministic replacement.

```bash
awk '/SPEC_LIVE_PATHS=\(/,/^\)/' ../../scripts/run-specs.sh | mustmatch not like "spec/entity/clingen-car-live.md
spec/entity/clingen-ldh-live.md"
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
