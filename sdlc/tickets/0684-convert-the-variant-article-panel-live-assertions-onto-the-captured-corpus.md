---
flow: build
priority: 8
---
# Convert the variant-article panel live assertions onto the captured corpus

Carried over from March ticket 684 when BioMCP moved to the sdlc
factory. The body below is March's, unchanged; it was already written to
stand alone. Work products from any earlier attempt:

    /home/ian/workspace/planning/biomcp/artifacts/684-convert-the-variant-article-panel-live-assertions-onto-the-captured-corpus
## Why

This is the second half of ticket 665, split after four aborts. Sibling ticket 683 captures
the seven-variant panel corpus and records the request-to-landmark mapping. This ticket
builds the strict-replay fixture on top of it and converts the live assertions.

665's original goal is unchanged: retire the live-provider dependency in
`spec/entity/variant-articles-live.md` by proving the same orchestration behavior against
recorded bytes.

## Scope

The replay fixture, Tier 2 and Tier 3 source tests, and the live-to-routine conversion.

Out of scope: capturing the corpus (683 owns it), and the frozen identity contract and its
fixture (682 owns that; see the ownership ruling below).

## Intermediate State

Live assertions leave `SPEC_LIVE_PATHS` only as their replacements land. A page keeps its
entry while any unconverted block remains, and a partially converted page is recorded as
partial — not as done.

## Fixture ownership — decided, do not re-litigate

`spec/fixtures/run-variant-article-identity-fixture.sh` currently serves two contracts, and
the frozen identity spec's assertions depend on its synthesized data.

**One fixture per contract.** This ticket's orchestration contract gets its **own** fixture
serving only real receipted captures. The existing identity fixture and its spec are
untouched. Re-deriving that frozen contract onto real bytes is ticket 682's job.

## Success Checklist

- [ ] Every converted assertion has a consumed Tier 2 request plan, a Tier 3 production
      decoder/orchestration proof over the real captures, and fixture-backed CLI proof where
      it claims presentation.
- [ ] The fixture replays 683's captures **byte-for-byte**. No response is constructed
      inline, reshaped, or trimmed to fit.
- [ ] The fixture dispatches on an explicit expected-request table and **fails loudly** on
      any path it was not told to serve. A fixture that answers unrecognized requests with a
      generic response converts a wrong request plan into a green test — this was review
      defect 4 on 665's first attempt.
- [ ] Positive, empty, degraded and not-attempted source states are each proved by their own
      recorded evidence and their own assertion.
- [ ] Each landmark PMID is attributed to binary-emitted route-stage evidence, so a provider
      absence cannot be read as a BioMCP pipeline loss. This is the existing contract's own
      language; preserve it.
- [ ] Exact counts against the captured corpus are asserted as exact. They are deterministic
      and they are the proof, not trivia — see ticket 681 and `planning/flows/build/05-verify.md`.
- [ ] Tier 2 and Tier 3 coverage lands **before** the live assertion is removed. Retiring a
      canary before its replacement exists reduces coverage rather than relocating it.
- [ ] No production behavior is changed to make a capture assert cleanly. Ticket 662's review
      caught exactly that and reverted it.
- [ ] `make lint`, `make test`, and `make spec` pass.

## Dependencies

Ticket 683 must land first — it owns the corpus and the mapping this ticket consumes.

## Notes

- Green gates are not evidence. 665's first attempt passed `make lint`, `make test` and
  `make spec` with 225 assertions on a fully synthetic fixture, and only code review caught
  it.
- If the corpus turns out to be insufficient for an assertion, that is a finding to report
  against 683 — not grounds to synthesize the missing response or weaken the assertion.

## Operator addendum — 2026-08-06: no CAR or LDH assertions on this panel

Ticket 683 originally listed six routes including ClinGen CAR and ClinGen LDH. That list
was wrong and has been struck; see 683's operator ruling for the verification.

All seven panel inputs are bare protein changes, which produce
`canonical_equivalence=inapplicable` with `applicable_identity_count=0`. CAR is therefore
never called and LDH — which requires CAR's resolved CAid — is never called either. The
shipped canary's own `recognized_routes` set
(`spec/fixtures/run-variant-articles-live-canary.sh:130-134`) confirms it: four logical
routes, no CAR, no LDH.

1. **Assert only the routes 683's mapping actually records.** Do not add a CAR or LDH row to
   the dispatch table, and do not add a capture for either under this ticket.
2. **The dispatch table's fail-loudly requirement covers this too.** If a request to a CAR
   or LDH host reaches the fixture during this panel's tests, that is a real regression in
   route gating and the test must fail rather than serve a stored body.
3. CAR and LDH proof lives in ticket 662's captures under
   `testdata/sources/clingen_allele_registry/` and `testdata/sources/clingen_ldh/`.
