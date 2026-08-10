---
flow: quickfix
priority: 10
---
# Reject unmapped NCI trial filters

NCI trial search currently accepts study type, sponsor, and update-date
filters without putting them in the request. When biomarker, mutation, and
criteria are supplied together, it silently chooses the first one.

## Provider contract

For `--source nci`, continue to support condition, intervention, facility,
mapped status, mapped phase, and a complete latitude/longitude/distance tuple.
Support exactly one of `--biomarker`, `--mutation`, or `--criteria` through the
NCI combined biomarker query field.

Reject `--study-type`, `--sponsor`, `--date-from`, and `--date-to` before
transport. Reject any request containing more than one of biomarker, mutation,
and criteria; do not concatenate them or choose one. Existing NCI rejections
for age, sex, sponsor type, results availability, unsupported phase/status, and
CTGov-only eligibility filters remain in force. A filter may become accepted
later only with a request-construction fixture proving its exact NCI mapping.

## Done when

- A table covers every public trial filter for NCI as mapped or rejected.
- Every rejected single filter and combination fails before a counting local
  transport sees a request.
- Request observations pin every supported NCI field and prove the selected
  biomarker-like value is sent exactly once.
- CLI, MCP, list/help, trial documentation, and provider capability text agree.
- CTGov behavior and shared validation remain unchanged.
- No routine test reaches NCI or MyDisease.

## Authorized test changes

The quickfix may restate NCI validation and construction expectations in
`src/entities/trial/search/mod.rs`, `src/entities/trial/search/nci.rs`,
`src/sources/nci_cts`, trial CLI tests, specs, and trial/source documentation.
Existing mapped status, phase, disease grounding, pagination, and CTGov tests
must remain covered.

The src line ceiling may rise by at most 100 lines.
