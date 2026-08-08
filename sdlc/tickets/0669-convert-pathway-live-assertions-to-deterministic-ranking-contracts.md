---
flow: build
priority: 5
---
# Convert pathway live assertions to deterministic ranking contracts

Carried over from March ticket 669 when BioMCP moved to the sdlc
factory. The body below is March's, unchanged; it was already written to
stand alone. Work products from any earlier attempt:

    /home/ian/workspace/planning/biomcp/artifacts/669-convert-pathway-live-assertions-to-deterministic-ranking-contracts
## Why
Ticket 645 found pathway alias normalization, ranking, capability, and source cards are local merge behavior but no corresponding receipted KEGG, Reactome, or WikiPathways responses exist.

## What
Add consumed search/detail plans for KEGG, Reactome, and WikiPathways; record receipted real alias/search/detail captures; pin production ranking/capability transforms and CLI cards; then convert `pathway.md` live blocks.

## Intermediate State
Pathway aliases and ranked source cards are covered locally by provider-faithful bytes rather than fixed live records.

## Scope

What is IN scope:
- `src/entities/pathway.rs`, the three source modules' tests/captures, and `spec/entity/pathway.md`.

What is OUT of scope:
- New pathway providers or ranking policy changes.

## Dependencies
- 651-enforce-receipt-backed-real-captures-for-tier-3-source-tests

## Success Checklist
- [ ] Each converted assertion has a consumed Tier 2 plan, receipt-backed real Tier 3 production decoder/orchestration proof, and fixture-backed CLI proof where it claims presentation.
- [ ] Only replaced live blocks leave `SPEC_LIVE_PATHS`; all retained local proof remains routine.
- [ ] A live test may be replaced by unit tests **only if** those unit tests fully cover the CLI-to-API-call transition and the parsing of a locally captured response. Everything on our side of the socket stays covered; only "does the provider serve this today" is dropped.
- [ ] Tier 2 and Tier 3 coverage lands **before** the live assertion is removed. Retiring a canary for a source with no unit tests reduces coverage rather than relocating it.
- [ ] Captures are real recorded responses with their capture date. A synthesised or edited fixture proves only that a parser agrees with itself — ticket 614's lesson, and the mechanism behind 650.
- [ ] Issues `605`, `606`, `612`, `614` and `636-make-semantic-scholar-canary-credential-routing-deterministic` close as the coverage that replaces them lands — not before, and not on the strength of the 2026-08-01 green run. A green run proves the provider is up today, which is exactly the property these canaries should stop asserting.
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
