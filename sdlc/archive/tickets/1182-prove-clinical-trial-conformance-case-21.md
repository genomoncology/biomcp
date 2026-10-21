---
flow: build
priority: 10
---
# Prove clinical-trial conformance case 21 at the product boundary

Archived as superseded by ticket 1183. Its local NCI public-boundary proof remains part of ticket 1183 acceptance; its former cross-repository handoff is no longer active.

## Goal

Focused, stable tests prove that BioMCP turns recorded NCI structured eligibility into complete JSON and readable Markdown. A malformed present eligibility value reaches the caller as a sanitized error.

## Current evidence

BioMCP `e8f02aa851bc05051d446fbf3e311ad9ef8a4e11` already serves `testdata/sources/nci_cts/get_nci_2023_04529_full_20260903.json` to the real binary in `tests/test_nci_filter_transport.py::test_nci_detail_executes_the_biodata_plan_through_the_real_cli`. That broad test verifies 36 criteria, source order, inclusion and exclusion headings, age, sex, healthy-subject state, and request shape. `detail_response_passes_untouched_success_bytes_to_biodata` separately proves the untouched internal byte handoff.

`src/entities/trial/get/tests.rs::nci_criterion_sorting_preserves_source_occurrence_identity_and_classification` verifies descriptions, source occurrence identities, order, and classifications. `src/sources/nci_cts/tests/parsing.rs::detail_response_rejects_identity_and_old_lenient_shapes` verifies sanitized BioData failures but does not exercise malformed eligibility through the public product process.

BioData conformance case 21 needs a stable, narrowly named BioMCP product proof to pin. BioData uses a different recorded fixture with 42 criteria. Do not claim the captures are identical.

## Required behavior

Provide independently runnable public-boundary success and malformed-input tests. Record their stable pytest node IDs in the completion record so the following BioData ticket can pin them. Reuse BioMCP's existing receipted NCI fixture and a loopback NCI server. Do not add or alter a provider fixture.

JSON contains every recorded eligibility description exactly once. It preserves each original row's one-based occurrence identity, classification, and order after stable numeric `display_order` sorting. Markdown contains every description in the same order and uses inclusion and exclusion headings according to classification.

Mutating a copied description changes JSON and Markdown. Mutating `display_order` changes presentation order while retaining each row's occurrence identity. Mutating `inclusion_indicator` changes the JSON classification and Markdown heading. The ordering proof uses a derived response whose array order disagrees with numeric `display_order`. An independent test oracle supplies the expected order.

A present non-object `eligibility`, a present non-array `eligibility.unstructured`, or any criterion with a missing, null, wrongly typed, blank, or out-of-range required member makes the command fail. Missing or null optional containers retain their existing absence behavior.

The public command exits with code 1. Standard output contains only the structured error. The error has `error.code` equal to `api`, message `API request to NCI Clinical Trials Search failed.`, source `NCI Clinical Trials Search`, recovery `Retry the remote source.`, and `_meta.not_found` equal to `false`. Inject unique provider, criterion, credential, and requested-identifier sentinels. Neither output stream contains a sentinel or partial trial content.

Preserve the existing focused guarantees for criterion ordering, classification, occurrence identity, untouched-byte handoff, and sanitized invalid-shape failures.

## Red proof

The two stable public-boundary selectors must collectively fail under four temporary production mutations in an isolated test copy: replace one criterion description with a constant, ignore `display_order`, force every classification to inclusion, and turn an unreadable present eligibility value into absence. Fixture sensitivity checks alone do not satisfy this proof. Do not commit the production mutations.

## Done, observably

- A stable successful selector proves the recorded JSON and Markdown behavior through the real CLI.
- A stable malformed selector proves the exact safe failure contract and no partial output.
- Description, order, classification, and malformed-as-absence mutations make the appropriate focused test fail.
- The existing Rust ordering/classification and invalid-shape tests pass.
- `make lint`, `make test`, and `make spec` pass under Node 22.

## Boundaries

No BioData change, provider request change, network request, fixture change, search change, eligibility meaning change, JSON shape change, Markdown wording change, second conformance registry, release, or publication belongs here. Preserve the documented limitation that the receipted capture came from a search envelope and supplies the trial record served by the local detail fixture. A following BioData ticket pins this ticket's exact integrated revision and selectors.
