---
flow: build
priority: 3
---
# Finish mixed CLI live-block conversion after source contracts land

Carried over from March ticket 673 when BioMCP moved to the sdlc
factory. The body below is March's, unchanged; it was already written to
stand alone. Work products from any earlier attempt:

    /home/ian/workspace/planning/biomcp/artifacts/673-finish-mixed-cli-live-block-conversion-after-source-contracts-land
## Why
Ticket 645 found `spec/surface/cli.md` to be mixed: its help, list, and static blocks are already routine proof, while each source-backed health or command block belongs to its source-specific deterministic replacement.

## What
Inventory source-backed blocks in `spec/surface/cli.md`, reuse the landed source contracts and receipts from all predecessor slices, and remove or split only those replaced live blocks so `SPEC_LIVE_PATHS` is accurate at assertion granularity. Preserve all local CLI help/list/rendering assertions in their current routine document.

## Intermediate State
The registry contains no CLI page solely because of source availability; the mixed CLI document remains a routine user-surface contract.

## Scope

What is IN scope:
- `spec/surface/cli.md`, `scripts/run-specs.sh::SPEC_LIVE_PATHS`, and any local fixture wiring required only to route converted CLI blocks.

What is OUT of scope:
- Changing CLI grammar, help/list behavior, health semantics, or removing an unconverted source assertion.

## Dependencies
- All source conversion tickets listed in frontmatter must be shipped first.

## Success Checklist
- [ ] Each converted assertion has a consumed Tier 2 plan, receipt-backed real Tier 3 production decoder/orchestration proof, and fixture-backed CLI proof where it claims presentation.
- [ ] Only replaced live blocks leave `SPEC_LIVE_PATHS`; all retained local proof remains routine.
- [ ] A live test may be replaced by unit tests **only if** those unit tests fully cover the CLI-to-API-call transition and the parsing of a locally captured response. Everything on our side of the socket stays covered; only "does the provider serve this today" is dropped.
- [ ] Tier 2 and Tier 3 coverage lands **before** the live assertion is removed. Retiring a canary for a source with no unit tests reduces coverage rather than relocating it.
- [ ] Captures are real recorded responses with their capture date. A synthesised or edited fixture proves only that a parser agrees with itself — ticket 614's lesson, and the mechanism behind 650.
- [ ] Nothing here closes an issue. The five live-canary reports that used
  to be listed at this point were folded into ticket 0885 and their files
  deleted; 0885 owns them now. Do not go looking for them, and do not treat
  a green live run as evidence — a green run proves the provider is up
  today, which is exactly the property those canaries should stop
  asserting.
- [ ] The Semantic Scholar credential dependency is proven by capturing the outgoing header. It cannot be proven by an anonymous 429 — measured intermittent at 2 of 4 attempts.
- [ ] `make lint`, `make test`, and `make spec` pass.

## Notes
- See `architecture/technical/live-spec-conversion-target.md` for the target boundary and per-path prerequisites.
- No FAQ entry is currently `watching`; FAQ #17's standard gates remain the proof lane.

## Operator addendum — 2026-08-05: observe the provider's shape before asserting against it

Two tickets landed assertions against provider data shapes that do not exist:

- **678** asserted a package at `.../PMC<id>.<v>/metadata/<v>.json`. That path 404s. The
  real layout is flat: `.../PMC7382263.1/PMC7382263.1.json`.
- **663** asserted that the first citing paper for PMID 22663011 has no PubMed ID, no DOI
  and no arXiv ID. That record has 100 citation rows and **zero** such papers.

Neither was a bad capture. In both cases a design imagined what the provider returns, a
local fixture was built to match the imagination, and the assertion confirmed the fixture.
The loop closed cleanly and proved nothing.

Note this is **not** covered by "captures are real recorded responses". Both failures were
in hand-built fixtures, which that constraint does not reach. A real capture sitting beside
an invented fixture still yields a green run that means nothing.

### Requirements

- [ ] Every assertion about a provider's data shape names the request that was issued and
      the value that came back, recorded in the design. No assertion is authored from an
      assumed schema, an API doc, or a remembered shape.
- [ ] Any locally-built fixture is derived from an observed real response. If the fixture
      serves a shape no real response produced, say why in the design and expect challenge.
- [ ] If the design's chosen record cannot exhibit the behavior being pinned, **change the
      record, not the assertion**. 663's provider-only citation row was real; the seed was
      wrong. A bounded search across a handful of candidate records found it.
- [ ] Where an assertion depends on a row's position in a response, do not rely on it.
      663's target rows sat at indexes 74, 88 and 96 — never first.
- [ ] No production behavior is changed to make a capture assert cleanly. Ticket 662's
      review caught exactly that: `verify_ldh_annotation` was altered so a capture could
      assert completeness, and the review reverted it.
- [ ] No fixture is synthesized, hand-edited, or reshaped to match our code. Tickets 614,
      650 and 652 each did this; 652's fixture carried invented PMIDs.
