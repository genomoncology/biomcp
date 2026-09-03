---
flow: build
priority: 7
---

# NCI eligibility text is always absent, because the reader calls as_str on an object

`get_trial` reads NCI eligibility at `src/entities/trial/get.rs:179-183`:

```rust
let criteria = resp
    .get("eligibility")
    .and_then(|v| v.as_str())
    .map(str::trim)
    .filter(|s| !s.is_empty());
```

NCI sends `eligibility` as an object, not a string. `as_str()` returns `None` on every record, the criteria are never read, and the code takes the `else` branch and writes a log line:

```rust
warn!(nct_id, "NCI CTS eligibility criteria not found in response");
```

The log line is wrong twice. The criteria are in the response. And a log line is not a report: the caller receives an absent field and no indication that a conversion failed, so a reader cannot tell "NCI published no criteria" from "BioMCP could not read the criteria NCI published".

A user asking whether they qualify for an NCI trial gets nothing, on every NCI trial, and the answer looks like the registry's silence rather than ours.

Measured against this repository's own recorded NCI capture, `testdata/sources/nci_cts/search_melanoma.json`. In record `NCT05929768`, `eligibility` is an object with two members: `structured`, an object, and `unstructured`, a list. The BioData lead's live query of `clinicaltrialsapi.cancer.gov/api/v2/trials` on 2026-09-02 saw the same shape.

Note that the criteria arrive as a list of entries rather than one block of prose, so this is not a matter of reading a different key name. It is a shape the reader was never written for. What the entries carry, and how they compose into the eligibility text the caller receives, is for the design stage to read off the payload and settle.

## This one is a type mismatch, not a name mismatch

Four sibling defects filed the same day are all the same shape: the reader asks for a key name the provider does not send. This one asks for the right key and mishandles its type. The outcome is identical, silent absence, which is why it travels with them. The cause is different, which is why it is its own ticket.

## Required behavior

An NCI trial reports the eligibility criteria its payload carries.

A payload shape the reader cannot handle is reported to the caller as a conversion failure. It is never a silent absence, and a log line is not a report.

## Done, observably

- Requesting the eligibility section for a trial whose recorded NCI payload carries an eligibility object yields non-empty eligibility text.
- A payload whose eligibility is a shape the reader cannot handle produces a conversion failure the caller can see, distinct from a payload that carries no eligibility at all.
- The assertion is made against a recorded capture, not against an object written by hand.
- The `warn!` at `src/entities/trial/get.rs:186` no longer fires on a payload that does carry criteria.

## The fixture, honestly

`testdata/sources/nci_cts/search_melanoma.json` carries the eligibility object and proves the shape. Two caveats, both worth knowing before the design stage starts.

It is classified `pending_verification` in `testdata/sources/capture-receipts.json`, so it has no provider receipt. The one receipted NCI capture was recorded minimized to six fields and carries no eligibility object. That gap has its own ticket, filed the same day, and closing it is not this ticket's job.

It is also a search response, and the code path this ticket fixes is the single-trial get. Whether the get endpoint nests eligibility identically is worth confirming from the payload rather than assumed. If the two differ, say so in the record.

## Where correct behavior is written

`sdlc/planning/clinical-trial-conformance/cases.json` in the BioData repository, case 21, "Eligibility text is read from the structure the provider sends". That file is the shared statement of correct behavior, held against both 0.9 and 1.0 so the two cannot drift.

The behavior is restated above in full, because an attempt runs in a worktree where that path resolves to nothing. ADR 0025's amendment of 2026-09-03 says the restatement is what carries the statement across, and a person reconciled the two when this ticket was filed. If the restatement above looks wrong, stop and say so rather than implementing something different.

## Boundary

Change only the NCI eligibility read in `src/entities/trial/get.rs` and its tests.

Do not change the ClinicalTrials.gov eligibility read in the same function. Its shape handling is correct.

Do not change `truncate_inline_text` or `ELIGIBILITY_MAX_CHARS`. Truncation behavior stays as it is.

Do not change age reading. NCI's structured eligibility also holds the age bounds, and those are a separate ticket filed the same day. Reading the criteria text here must not also start populating the age range, or two tickets land one change and the record for the other is a claim about work nobody did.

Do not touch interventions, study type, enrollment or the stop reason. Each is its own ticket.

## History

Found 2026-09-03 by the BioData lead while auditing the conformance cases, verified here independently against this repository's own capture and source the same day. Split out of ticket 1132, which bundled five behaviors and was superseded for that reason.
