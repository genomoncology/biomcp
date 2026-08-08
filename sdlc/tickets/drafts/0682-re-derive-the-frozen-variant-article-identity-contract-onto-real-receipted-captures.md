---
flow: build
priority: 4
---
# Re-derive the frozen variant-article identity contract onto real receipted captures

Carried over from March ticket 682 when BioMCP moved to the sdlc
factory. The body below is March's, unchanged; it was already written to
stand alone. Work products from any earlier attempt:

    /home/ian/workspace/planning/biomcp/artifacts/682-re-derive-the-frozen-variant-article-identity-contract-onto-real-receipted-captures
## Why

`spec/entity/variant-article-identity.md` is a frozen shipped contract whose assertions are
proved entirely against a **synthesized** fixture. `spec/fixtures/run-variant-article-identity-fixture.sh`
constructs its rows, PubTator documents, ClinGen CAR identifiers, LDH annotations and
scenario modes inline rather than replaying recorded provider bytes.

A contract asserted against invented bytes proves only that our code agrees with itself.
This is the defect the whole live-conversion effort exists to remove — the same family as
614 (synthesized fixture), 650 (hand-edited ERepo `@id`), 652 (manufactured CSpec criteria
with invented PMIDs) and 665's first attempt.

This is **pre-existing debt**, not created by ticket 665. It surfaced when 665 needed the
same shared fixture and could not replace its bytes without breaking these assertions.
Ticket 665 was ruled to split fixture ownership and leave this contract untouched, which
unblocked 665 but deliberately left this unaddressed. Filing it so the split is not mistaken
for an endorsement.

## Scope

Re-derive the identity contract's evidence onto real, receipted captures, or record with
justification which scenarios cannot be.

Out of scope: ticket 665's new seven-variant orchestration contract and its separate
fixture; any change to what the identity contract *means*.

## Intermediate State

None.

## Success Checklist

- [ ] Every identity scenario that a real provider response can exhibit is proved against
      committed `real_and_receipted` captures, replayed byte-for-byte.
- [ ] Captures pass `tools/check-source-capture-receipts.py` and carry their capture date.
- [ ] The fixture serves recorded bytes through an explicit expected-request dispatch table
      that **rejects** unrecognized paths. A fixture that answers everything cannot prove
      the request was right.
- [ ] Scenarios that real data genuinely cannot exhibit — deliberately degraded, empty, or
      error states with no natural analogue — are enumerated with the reason each one
      resists capture. This list is the deliverable for the remainder, not a silent gap.
- [ ] The contract's asserted behavior does not change. If a real capture contradicts a
      current assertion, that is a finding to report, not an assertion to edit.
- [ ] No production behavior is changed to make a capture assert cleanly. Ticket 662's
      review caught exactly that and reverted it.
- [ ] `make lint`, `make test`, and `make spec` pass.

## Dependencies

None strictly, but sequence **after** 665 lands. 665 is building the real-capture corpus and
strict-replay fixture pattern for the same providers; this ticket should reuse that corpus
and that pattern rather than inventing a second approach.

## Notes

- Capture route precedent, including the Semantic Scholar case that has no loopback
  recording path: ticket 663's `testdata/sources/semantic_scholar/pmid20516115-*.json`,
  served via `BIOMCP_S2_BASE` (`src/sources/semantic_scholar.rs:14`).
- Bound the corpus by what the assertions exercise, not by what a request inventory happens
  to record.
- Green gates are not evidence. 665's first attempt passed `make lint`, `make test` and
  `make spec` — 225 spec assertions — entirely on invented data.
