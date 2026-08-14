---
flow: build
priority: 10
---
# Reject inapplicable adverse-event search filters

Adverse-event search accepts several valid-looking flags that the selected
route does not use. Recall silently drops reaction and seriousness filters,
FAERS silently drops recall classification, and device search turns every
`--serious <value>` into the broader Death-or-Injury query. A successful
command must not imply that an ignored biomedical filter was applied.

Validate the complete query-type and source compatibility boundary before any
provider request. FAERS, VAERS-only, recall, and device searches must reject
every supplied flag they cannot honor. Preserve the existing combined-source
behavior: `--source all` may run FAERS while reporting an incompatible VAERS
branch as not requested.

The accepted route matrix is explicit:

- FAERS accepts a drug query, reaction, outcome, seriousness, both dates,
  suspect-only, sex, both age bounds, reporter, limit, offset, and count.
  Classification and all device fields are invalid. Count requires explicit
  `--source faers`; `--source all --count` is rejected rather than silently
  becoming a FAERS-only response.
- VAERS-only accepts a vaccine query and limit. Limit bounds the returned top
  reaction rows. Offset, count, every other FAERS filter, classification, and
  all device fields are invalid.
- Recall accepts a drug query, classification, limit, and offset. Every other
  filter is invalid.
- Device accepts device, manufacturer, product code, date-from, typed
  seriousness, limit, and offset. A drug or positional query and every other
  filter are invalid.
- Combined `--source all` applies limit to both branches. FAERS-only filters or
  a nonzero offset run FAERS and leave VAERS visibly not requested. Count is
  not a combined operation and is rejected as described above.

For device search, bare `--serious` and `--serious any` mean Death or Injury,
`--serious death` means Death only, and `--serious injury` means Injury only.
FAERS-only seriousness values are invalid for device searches. Help, query
summaries, Markdown, and JSON must describe the effective filter truthfully.

## Done when

- Inapplicable search flags return a structured argument error before provider contact.
- Every accepted flag changes the selected request plan exactly as advertised.
- Device seriousness preserves the three settled meanings above.
- Provider predicates are exact: any is Death-or-Injury, death is Death only, and injury is Injury only; summaries and JSON describe the same choice.
- Existing valid FAERS, combined FAERS/VAERS, recall, and device searches remain compatible.

## Authorized test changes

The design may add or restate assertions in `src/cli/adverse_event/tests.rs`,
the native tests in `src/entities/adverse_event.rs`, and a focused process
contract under `tests/` for pre-contact rejection and output truthfulness.
