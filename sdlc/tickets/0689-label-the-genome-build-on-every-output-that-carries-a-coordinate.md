---
flow: build
priority: 9
deps: [0898]
---
# Label the genome build on variant detail output

**Rescoped 2026-08-10 after eight refusals.** This ticket was "label every
emitted coordinate on every route in both JSON and Markdown." That is three
tickets, and the design stage was refused for saying so. It is now slice one
of three:

- **0689 (this one)** — `get variant` detail routes, JSON only.
- **0899** — gene and search surfaces, JSON only.
- **0900** — rendered/Markdown output, across all of them.

Do not widen this ticket back out. Coverage of the other surfaces is not
missing; it is scheduled.

## Why

Ticket 687 made the *input* side build-aware: hand BioMCP a coordinate and it
tells you which build it resolved against. The *output* side was never
covered. Measured against the merged binary (`268ea626`):

| input | emitted coordinate | `genome_build` |
|---|---|---|
| `chr7:g.140453136A>T` | `chr7:g.140453136A>T` | `GRCh37` |
| `rs113488022` | `chr7:g.140453136A>T` | **`null`** |
| `rs121913529` | `chr12:g.25398284C>T` | **`null`** |
| `BRAF V600E` | `chr7:g.140453136A>T` | **`null`** |

Every unlabeled coordinate there is GRCh37, because MyVariant defaults to
hg19. They are correct values wearing no label.

A caller looks up `rs113488022`, gets `chr7:g.140453136A>T`, and carries it
into a report or a pipeline that assumes GRCh38. Nothing in our output said
which map it came from. That is how the original bug propagated.

## Scope

In scope — `get variant`, `--json` only:

- The rsID route.
- The gene+protein route (`BRAF V600E`).
- The unqualified transcript-HGVS detail branch, which can still return
  `answering_build = None`. Named by the 2026-08-09 design review after
  reading the code; do not rediscover it.

Out of scope, and not a gap:

- `search variant`, `Gene`, `GeneSearchResult`,
  `VariantNormalizationServiceResult` — those are 0899.
- Rendered/Markdown output — that is 0900.
- **Changing which build any route resolves against.** This ticket adds a
  label and moves nothing. Whether GRCh38 should become the default is an
  open operator question, decided elsewhere.

## Done when

- [ ] `get variant rs113488022 --json` reports `genome_build: "GRCh37"`, not
      null.
- [ ] `get variant 'BRAF V600E' --json` reports the build.
- [ ] The unqualified transcript-HGVS branch reports a build, or states in
      the output why it cannot. An absent field is not an acceptable answer.
- [ ] Every assertion is backed by a receipted capture, not a synthesized
      fixture.
- [ ] Coordinate values are byte-identical to `main` for the four inputs in
      the table above. Prove it with a before/after comparison. This ticket
      adds a label and moves nothing.
- [ ] `make lint`, `make test`, and `make spec` pass.

## Tests you are authorized to restate

These files assert today's unlabeled output. The design stage must update
them, and doing so is expected rather than a traceability violation:

    src/cli/variant/tests.rs
    src/entities/variant/get/tests.rs
    src/cli/tests/outcome.rs
    skills/examples/get-variant-rs113488022-all.json
    skills/schemas/variant.json

Change only what the new label requires. Every other assertion in those files
keeps checking exactly what it checks today.

If the implementation needs a test file **not** on this list, that is a
signal the ticket is wrong — stop and say so in the design rather than
editing it.

## Dependencies

0898 must land first. It makes these structs constructible without naming
every field; without it a `code:` commit here cannot compile and cannot
legally fix itself. That is what refused attempts two through eight.

## History worth keeping

Six attempts' design evidence is preserved under `attempt/0689-*` tags. The
2026-08-09 review repaired the authored PTEN assertions (commit `18d64f77` on
that claim branch): coordinate stability pinned, meaningful label required,
receipted request matched. Those repairs are worth reading before designing
again — but note they span all three slices, so only the variant-detail parts
belong here.

The src line ceiling may rise by at most 60 lines.
