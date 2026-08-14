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

For device search, bare `--serious` and `--serious any` mean Death or Injury,
`--serious death` means Death only, and `--serious injury` means Injury only.
FAERS-only seriousness values are invalid for device searches. Help, query
summaries, Markdown, and JSON must describe the effective filter truthfully.

## Done when

- Inapplicable search flags return a structured argument error before provider contact.
- Every accepted flag changes the selected request plan exactly as advertised.
- Device seriousness preserves the three settled meanings above.
- Existing valid FAERS, combined FAERS/VAERS, recall, and device searches remain compatible.

## Authorized test changes

The design may add or restate assertions in `src/cli/adverse_event/tests.rs`,
the native tests in `src/entities/adverse_event.rs`, and a focused process
contract under `tests/` for pre-contact rejection and output truthfulness.
