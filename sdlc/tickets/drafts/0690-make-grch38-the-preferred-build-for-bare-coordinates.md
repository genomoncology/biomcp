---
flow: build
priority: 8
deps: ["0900"]
---
# Make GRCh38 the preferred build for ambiguous bare coordinates

Ian chose GRCh38 as the modern default on 2026-08-07. This ticket changes
preference under ambiguity; it does not remove GRCh37 lookup.

## Resolution contract

- Probe both GRCh38 and GRCh37 as today.
- GRCh38-only input returns the GRCh38 record.
- GRCh37-only input still returns the GRCh37 record.
- A coordinate with records in both builds returns the GRCh38 record and
  retains the GRCh37 record as the competing candidate.
- Every coordinate and candidate is labeled by 0689, 0899, and 0900.

Use this precedence:

1. an explicit command/MCP assembly value;
2. BIOMCP_DEFAULT_ASSEMBLY, accepting grch37/hg19 or grch38/hg38;
3. GRCh38 preference.

Do not add a configuration-file subsystem. The environment setting is the
durable process-level escape hatch; explicit input still wins.

## Done when

- A GRCh37-only fixture still resolves and is labeled GRCh37.
- A GRCh38-only fixture still resolves and is labeled GRCh38.
- chr10:g.87933119A>C returns PTEN/GRCh38 rs759485888 and reports the
  GRID1/GRCh37 rs1212585646 record as the competing candidate.
- --assembly hg19 and BIOMCP_DEFAULT_ASSEMBLY=grch37 restore the old
  preference; explicit CLI/MCP input overrides the environment.
- Invalid environment values fail clearly at startup/use rather than silently
  selecting a build.
- Help, JSON/MCP docs, user docs, and changelog state the precedence and
  breaking behavior.

Use targeted GRCh37-only, GRCh38-only, and collision fixtures for the blocking
gate. A broad live coordinate sample is optional measurement, not a committed
82-capture correctness corpus.

## Authorized test changes

Design commits may restate the collision preference tests landed by 0687,
variant CLI/MCP configuration tests, docs, and changelog. The coordinate
identity assertions for non-collision cases remain unchanged.

This is a pre-1.0 breaking behavior change and belongs in the next minor
release, not an unnoticed patch.

The src line ceiling may rise by at most 120 lines.
