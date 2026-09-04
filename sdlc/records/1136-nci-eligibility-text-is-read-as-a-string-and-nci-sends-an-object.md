---
flow: build
priority: 8
---

# NCI eligibility objects are silently discarded by a string-only reader

`get_trial` reads NCI eligibility at `src/entities/trial/get.rs`, in `get`:

```rust
let criteria = resp
    .get("eligibility")
    .and_then(|v| v.as_str())
    .map(str::trim)
    .filter(|s| !s.is_empty());
```

The receipted NCI record sends `eligibility` as an object, not a string.
`as_str()` returns `None` for that supported shape, the criteria are never read,
and the code takes the `else` branch and writes a log line:

```rust
warn!(nct_id, "NCI CTS eligibility criteria not found in response");
```

The log line is wrong twice. The criteria are in this response. And a log line
is not a report: the caller receives an absent field and no indication that a
conversion failed, so a reader cannot tell "NCI published no criteria" from
"BioMCP could not read the criteria NCI published."

A user asking whether they qualify for an NCI trial with this object shape gets
nothing, and the answer looks like the registry's silence rather than ours. The
ticket does not claim every NCI trial publishes eligibility.

Measured against this repository's receipted unrestricted NCI capture,
`testdata/sources/nci_cts/get_nci_2023_04529_full_20260903.json`. Its trial
record carries `eligibility.structured` and 36 `eligibility.unstructured`
entries. Each unstructured entry has a string `description`, integer
`display_order`, and boolean `inclusion_indicator`; the first 19 are inclusion
criteria and the final 17 are exclusions.

The criteria therefore arrive as typed entries rather than one block of prose.
This is not a different key name, and blindly joining descriptions would erase
the distinction between who qualifies and who is excluded. The composition
contract is settled below before implementation begins.

## This one is a type mismatch, not a name mismatch

Ticket 1132 has completed the sibling field-name repairs for interventions,
age, study type, enrollment, and stop reason. This ticket asks for the correct
`eligibility` key but mishandles its type, so it remains a separate conversion
outcome.

## Required behavior

An NCI trial reports the eligibility criteria its payload carries, preserving
which criteria are inclusions and which are exclusions.

Implement one NCI-only helper in `src/entities/trial/get.rs` returning
`Result<Option<String>, BioMcpError>`. Missing or null `eligibility`, or a valid
eligibility object whose `unstructured` member is missing, null, or an empty
array, is checked textual absence and returns `Ok(None)`.

When `unstructured` is present, it must be an array of objects. Every entry must
carry a non-blank string `description`, an integer `display_order`, and a boolean
`inclusion_indicator`; extra provider fields are ignored. A non-object
`eligibility`, non-array `unstructured`, or unreadable entry is a caller-visible
`BioMcpError::Api` with `api: "nci_cts"`. The message names the invalid
eligibility structure but contains no payload or criterion text. Malformed
input never becomes absence and never returns a partial criteria list.

Sort entries by ascending `display_order`, stably preserving provider array
order for equal values. Render contiguous groups in that order with the exact
headings `Inclusion Criteria:` and `Exclusion Criteria:` and one `- ` bullet per
description. Repeat a heading if the indicator changes again. Preserve
description-internal line breaks. Apply the existing `truncate_inline_text` to
the complete rendered text once, after sorting, grouping, and labeling; keep
the existing 12,000-character bound and suffix behavior unchanged.

## Done, observably

- The receipted NCI record's 36 criteria render in `display_order`: 19 under
  `Inclusion Criteria:`, then 17 under `Exclusion Criteria:`. Tests pin the first
  inclusion clause, the first exclusion clause ("Definitive clinical or
  radiologic evidence of metastatic disease"), the last exclusion clause, and
  their order.
- Missing/null eligibility and valid structured-only, missing/null
  `unstructured`, and empty `unstructured` cases return `Ok(None)`.
- Wrong eligibility or `unstructured` containers and malformed individual
  entries return the stated `BioMcpError::Api`, with no criterion content in the
  error. Tests cover each class and prove no partial output.
- Sorting is stable for equal `display_order`, heading transitions are explicit,
  internal description line breaks survive, and truncation occurs only after
  the labeled text is composed.
- The positive assertion loads `/data/0` from the receipted capture rather than
  constructing the provider record by hand. A local HTTP fixture serves that
  same record as the raw detail response so the public NCI get path with the
  eligibility section is exercised without claiming the search envelope is a
  detail capture.
- The misleading NCI eligibility `warn!` is removed. Checked absence remains a
  successful response; malformed presence is the caller-visible error.

## The fixture, honestly

`testdata/sources/nci_cts/get_nci_2023_04529_full_20260903.json` is a
`real_and_receipted`, unrestricted NCI search capture and carries the complete
eligibility object at `/data/0`. It is the primary acceptance fixture.

The receipt is from `/trials?size=1`, while production get calls
`/trials/{nct_id}` and expects a raw trial object. This ticket does not claim the
capture directly attests the detail endpoint's envelope. The conversion helper
consumes a trial record independent of transport, and the get-path test serves
the receipted `/data/0` record as the raw detail response. Record this
limitation honestly; do not make a live request or add an unreceipted detail
fixture merely to erase it.

## Where correct behavior is written

`sdlc/planning/clinical-trial-conformance/cases.json` in the BioData repository, case 21, "Eligibility text is read from the structure the provider sends". That file is the shared statement of correct behavior, held against both 0.9 and 1.0 so the two cannot drift.

The behavior is restated above in full, because an attempt runs in a worktree where that path resolves to nothing. ADR 0025's amendment of 2026-09-03 says the restatement is what carries the statement across, and a person reconciled the two when this ticket was filed. If the restatement above looks wrong, stop and say so rather than implementing something different.

## Boundary

Change only the NCI eligibility read in `src/entities/trial/get.rs`, its tests,
and the one shared fixture-inventory declaration required by ticket 1126. Add
`testdata/sources/nci_cts/get_nci_2023_04529_full_20260903.json` to
`fixture_key_contract.on_disk` in
`testdata/sources/capture-receipts.json` with selector `/data/*` and endpoint
`nci`. The capture is already receipted; do not alter its bytes, receipt, attestor
role, or NCI top-level evidence limitation. No other provenance edit is
authorized.

Do not change the ClinicalTrials.gov eligibility read in the same function. Its shape handling is correct.

Do not change `truncate_inline_text` or `ELIGIBILITY_MAX_CHARS`. Truncation behavior stays as it is.

Do not change the completed ticket 1132 mappings for structured age,
interventions, study type, enrollment, or stop reason. Do not read any value from
`eligibility.structured` here.

## History

Found 2026-09-03 by the BioData lead while auditing the conformance cases and
verified here independently. Ticket 1132 later landed the sibling name-mapping
fixes; ticket 1126 supplied the receipted unrestricted capture and fixture
contract used here. This eligibility shape repair remains the next unblocked
clinical-data outcome before ticket 1138 adds the broader code-key guard.

## Completed 2026-09-03

The NCI get path now validates the provider's structured eligibility entries,
orders them stably by `display_order`, renders explicit inclusion/exclusion
groups, and truncates only the completed text. Missing content remains absence;
malformed present content is a payload-free `nci_cts` API error. The positive
parser and public-get-path tests consume `/data/0` from the receipted search
capture; as documented above, this proves conversion of the expected raw detail
record shape without claiming that capture attests the detail endpoint envelope.

Independent design review: ACCEPT. Independent code review: ACCEPT.

Repository gates passed: `make lint`; `make test` (846 passed, 3 skipped, plus
strict documentation build); and `make spec` (all declared mustmatch and static
contract batches passed).
