---
flow: build
priority: 9
---
# Label the genome build on every output that carries a coordinate

Carried over from March ticket 689 when BioMCP moved to the sdlc
factory. The body below is March's, unchanged; it was already written to
stand alone. Work products from any earlier attempt:

    /home/ian/workspace/planning/biomcp/artifacts/689-label-the-genome-build-on-every-output-that-carries-a-coordinate
## Why

Ticket 687 made the *input* side build-aware: if you hand BioMCP a coordinate, it now tells
you which build it resolved against. The *output* side was not covered, and every route that
emits a coordinate without having been given one still emits it unlabeled.

Measured against the merged binary (`268ea626`):

| input | emitted coordinate | `genome_build` |
|---|---|---|
| `chr7:g.140453136A>T` | `chr7:g.140453136A>T` | `GRCh37` |
| `rs113488022` | `chr7:g.140453136A>T` | **`null`** |
| `rs121913529` | `chr12:g.25398284C>T` | **`null`** |
| `BRAF V600E` | `chr7:g.140453136A>T` | **`null`** |
| `search variant --gene PTEN` | coordinates in results | **no build field at all** |

Every one of those unlabeled coordinates is GRCh37, because MyVariant's default is hg19.
They are correct values wearing no label.

This is the same failure this whole effort exists to remove. A caller looks up `rs113488022`,
gets `chr7:g.140453136A>T`, and carries that coordinate somewhere else — a report, a
pipeline, a spreadsheet, another tool that assumes GRCh38. Nothing in our output said which
map it came from. That is exactly how the original bug propagated, and `search variant` is
the more dangerous of the two because search is how people *discover* variants they do not
already have coordinates for.

## Scope

- Label the build on every emitted genomic coordinate, whatever route produced it: rsID,
  gene+protein, search results, and any renderer or JSON field carrying a coordinate.
- Cover both `--json` and rendered/markdown output. A field that only appears in JSON leaves
  the human-facing path unlabeled.
- Where a route's build is fixed by the provider default rather than chosen by the caller,
  say so explicitly rather than leaving the field absent.

Out of scope: changing which build any route resolves against. This ticket adds labels; it
does not move defaults. Whether GRCh38 should become the default is an open operator
question and is not decided here.

## Success Checklist

- [ ] `get variant rs113488022 --json` reports `genome_build: "GRCh37"`, not null.
- [ ] `get variant 'BRAF V600E' --json` reports the build.
- [ ] `search variant --gene PTEN --json` carries a build on every result row that has a
      coordinate.
- [ ] The rendered (non-JSON) output names the build wherever it shows a coordinate.
- [ ] No route silently omits the field. If a route genuinely cannot know the build, it says
      so in the output rather than emitting an absent field.
- [ ] Every assertion is backed by a receipted capture, not a synthesized fixture.
- [ ] The coordinate values themselves are unchanged from `main` — this ticket adds a label
      and moves nothing. Prove it with a before/after comparison on at least the four inputs
      in the table above.
- [ ] `make lint`, `make test`, and `make spec` pass.

## Dependencies
None. 687 has merged; this builds on the `GenomeBuild` plumbing it landed.

## Notes
Found during post-merge verification of 687, not by a failing test — nothing in the suite
asserts that an emitted coordinate carries a build.

## Bound by the 2026-08-09 design review (run 23-16-51-010d)

The refused design covered selected MyVariant routes and deferred the
rest — a scope cut this ticket does not permit. Actual code
inspection named the surfaces the next design MUST cover; do not
rediscover them:

- `Gene` and `GeneSearchResult` serialize `genomic_coordinates`
  without a build.
- `VariantNormalizationServiceResult` serializes
  `genomic_descriptions` without a build.
- The unqualified transcript-HGVS detail branch can still return
  `answering_build = None`.

The review repaired the authored PTEN assertions (commit 18d64f77 on
the claim branch, preserved under attempt tags): coordinate
stability pinned, meaningful Markdown label required, receipted
request matched. The repaired suite is red only on the six stated
missing-label failures — the next design starts from those tests.
