---
flow: build
priority: 8
deps: ["0900", "0933"]
---
# Make GRCh38 the preferred build for ambiguous bare coordinates

Ian approved GRCh38 as the modern default on 2026-08-07. This ticket changes
preference under ambiguity; it does not remove GRCh37 lookup.

## Resolution contract

- Probe both GRCh38 and GRCh37 as today.
- GRCh38-only input returns the GRCh38 record.
- GRCh37-only input still returns the GRCh37 record.
- A coordinate with records in both builds returns the GRCh38 record and
  retains the GRCh37 record as the competing candidate.
- Every coordinate and candidate is labeled by 0950, 0899, and 0900.

Use this precedence:

1. an explicit command or typed-MCP assembly value;
2. `BIOMCP_DEFAULT_ASSEMBLY`, accepting `grch37`/`hg19` or `grch38`/`hg38`;
3. GRCh38 preference.

Do not add a configuration-file subsystem. Resolve the environment value on
the first variant command that needs assembly preference, before constructing
any client or transport request. An invalid value returns the normal typed
configuration error then; unrelated commands such as `version` continue to
work. Explicit input still wins over a valid environment value.

The raw CLI and raw MCP route use `get variant ... --assembly <value>`. Add an
optional `assembly` field only to the typed MCP `get variant` branch, with the
four accepted enum spellings above; it maps to the same CLI argument. Do not
add assembly to unrelated typed entities. Any other variant route that already
accepts bare coordinates must consume the same resolved preference rather than
creating a second default.

## Done when

- A GRCh37-only fixture still resolves and is labeled GRCh37.
- A GRCh38-only fixture still resolves and is labeled GRCh38.
- `chr10:g.87933119A>C` returns PTEN/GRCh38 `rs759485888` and reports the
  GRID1/GRCh37 `rs1212585646` record as the competing candidate.
- `--assembly hg19` and `BIOMCP_DEFAULT_ASSEMBLY=grch37` restore the old
  preference; explicit CLI/raw-MCP/typed-MCP input overrides the environment.
- Every invalid environment spelling fails before transport on a variant
  command, while a process-level `version` case proves unrelated use works.
- Help, JSON/MCP schema and examples, user docs, and changelog state the same
  precedence and breaking behavior.

Use targeted GRCh37-only, GRCh38-only, and collision fixtures for the blocking
gate. A broad live coordinate sample is optional measurement, not a committed
82-capture correctness corpus.

## Authorized test changes

Design commits may restate the collision preference tests landed by 0687,
variant CLI/MCP configuration tests, the typed catalog from 0933, docs, and
changelog. Coordinate identity assertions for non-collision cases remain
unchanged.

This is an approved pre-1.0 breaking change. It may land without changing a
version, but the first public release containing it must be v0.9.0 or later;
ticket 0951 enforces that release boundary.

The src line ceiling may rise by at most 140 lines.
