---
flow: build
priority: 8
---
# Make GRCh38 the preferred build for bare coordinates

Carried over from March ticket 690 when BioMCP moved to the sdlc
factory. The body below is March's, unchanged; it was already written to
stand alone. Work products from any earlier attempt:

    /home/ian/workspace/planning/biomcp/artifacts/690-make-grch38-the-preferred-build-for-bare-coordinates
## Why

Operator decision (Ian, 2026-08-07): GRCh38 has been the current human reference for long
enough that it should be what BioMCP reaches for first. Today a bare coordinate resolves
GRCh37-first, inherited from MyVariant's `hg19` default rather than chosen.

## Read this before designing: "default GRCh38" must not mean "only GRCh38"

Measured 2026-08-07 against live MyVariant, 956 currently-working GRCh37 coordinates in the
PTEN/GRID1 region, asked what a naive flip would do:

| outcome if we simply switched the request to `assembly=hg38` | count | share |
|---|---|---|
| **breaks** — no GRCh38 record, previously-working lookup becomes not-found | 909 | 95.1% |
| **silently changes answer** — different rsID at the same coordinate | 44 | 4.6% |
| unchanged | 3 | 0.3% |

A naive flip breaks **95%** of working lookups. That is not a default change, it is a
removal of GRCh37 support.

The reason is that most catalogued variants exist at a coordinate in one build only. Being
"the newer reference" does not make GRCh38 a superset — the coordinates renumbered, so an
old coordinate usually names nothing in the new build.

So this ticket changes **preference under ambiguity**, not which builds we consult:

- A bare coordinate that resolves in **GRCh38 only** → GRCh38. (Unchanged from today.)
- A bare coordinate that resolves in **GRCh37 only** → GRCh37, still resolves, still labeled.
  **Must not break.** This is the 95%.
- A bare coordinate that resolves in **both** → return the **GRCh38** record, and surface the
  GRCh37 identity as the competing candidate. This is the actual behavior change, and it is
  the exact inverse of what 687 shipped.

Under that design the blast radius is the 44 collisions — 4.6% — every one of which already
emits `build_ambiguous: true` and names both candidates. Nothing silently breaks.

## Scope

- Invert the collision tie-break from GRCh37-preferred to GRCh38-preferred.
- Keep probing both builds. Do not reduce the set of builds consulted.
- Give callers a way to pin the old behavior globally, not just per-invocation. `--assembly
  hg19` already covers one call; someone with existing GRCh37 pipelines needs a durable
  setting. Design decides the mechanism (config file, env var); it must be discoverable in
  `--help` and documented.
- Update docs, help text, and the changelog. **This is a user-visible breaking change** —
  the same input returns a different record for the ambiguous cases. It needs a changelog
  entry that says so plainly and names the escape hatch.

Out of scope: the request-count/latency shape of dual probing. If preferring GRCh38 makes
the common GRCh37-only case cost an extra round trip, record the measurement and file it
separately rather than optimising here.

## Success Checklist

- [ ] A GRCh37-only bare coordinate still resolves, labeled GRCh37. Prove it across a sample
      of at least 50 of the 909 measured GRCh37-only coordinates — not one example.
- [ ] A GRCh38-only bare coordinate still resolves, labeled GRCh38.
- [ ] `chr10:g.87933119A>C` now returns the **PTEN/GRCh38** record (`rs759485888`), with
      `build_ambiguous: true` and the GRID1/GRCh37 identity (`rs1212585646`) listed as the
      competing candidate. This is the inverse of the assertion 687 landed; updating it is
      the point of this ticket, and it is the one place a shipped assertion legitimately
      changes meaning.
- [ ] `--assembly hg19` still pins GRCh37 for a single call.
- [ ] The durable GRCh37 setting works, is discoverable in `--help`, and is documented.
- [ ] Changelog records the breaking change, who it affects, and how to restore prior
      behavior.
- [ ] `make lint`, `make test`, and `make spec` pass.

## Dependencies
- 689-label-the-genome-build-on-every-output-that-carries-a-coordinate

689 must land first. Flipping which build wins while some outputs still omit the build label
means a caller's answer changes with nothing in the output saying why. Label first, then flip.

## Notes
Interacts with release 499 (v0.8.26, currently draft): this is a breaking behavior change and
should not ride a patch release unnoticed. Version and release sequencing is an operator call.
