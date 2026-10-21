---
flow: build
priority: 5
---

# Preserve working drug sections when DDInter is unavailable

Status: implemented; awaiting code review

## Outcome

A multi-section drug get keeps and renders the sections that completed when the
DDInter-backed `interactions` section is unavailable. The interaction failure is
represented by the shared typed section-outcome vocabulary instead of aborting
the whole card.

## Current facts

Reproduced on 2026-09-04 with the repository build
`./target/debug/biomcp` (`biomcp 0.9.0-dev.6`) and an explicit empty
`BIOMCP_DDINTER_DIR`:

```text
$ BIOMCP_DDINTER_DIR=<empty-directory> ./target/debug/biomcp get drug mercaptopurine all
Error: Source unavailable: DDInter is not available. Review source configuration and retry.
exit 1; stdout 0 bytes
```

JSON fails the same way: both `get drug mercaptopurine interactions` and
`get drug mercaptopurine label targets interactions` exit 1 and return only the
242-byte structured `source_unavailable` error, with no drug fields.

Under the same missing-DDInter setup, the individual `label`, `safety`,
`targets`, `approvals`, `regulatory`, `shortage`, `indications`, and `civic`
commands all exited 0 in this checkout. This establishes that DDInter, rather
than base drug resolution, caused the reproduced aggregate failure. It does not
make those live upstream results fixtures for the eventual tests.

With the repository's valid DDInter fixture, the same mercaptopurine
`interactions` request has zero structured DDInter rows but does carry usable
OpenFDA label interaction text (allopurinol and warfarin precautions). Drug
interactions are therefore additive evidence, not DDInter-only evidence: if
DDInter fails and this label text survives, the correct typed state is
`degraded` with `OpenFDA label` credited; `unavailable` is correct only when no
interaction evidence survives. Successful DDInter rows can also carry matched
DrugBank descriptions copied from the resolved MyChem/DrugBank base, and the
current provenance code credits `DrugBank` when those descriptions survive.
Canonicalizing this section must retain that existing attribution rather than
silently narrowing the provider inventory to DDInter and OpenFDA.

The immediate owner is `populate_common_sections` in
`src/entities/drug/get.rs`: it calls
`interaction_report_from_base(...).await?` before applying targets and
indications, so one DDInter error returns early and discards the already
resolved base/label and every later result. By contrast, the gene and protein
get paths already convert optional-source errors into `SectionOutcome` values
and continue.

Two claims in the original report were inaccurate:

- Drug `section_outcomes` currently has only `approvals`, `safety`, `targets`,
  `indications`, and `civic`; it does **not** have `interactions`.
  `src/entities/source_state_registry.rs` classifies drug interactions as a
  local selector with no canonical outcome key.
- Drug `all` expands to `label`, `regulatory`, `safety`, `shortage`, `targets`,
  `indications`, `interactions`, and `civic`. It deliberately does not include
  `approvals`, so approvals are not one of the sections discarded by the
  reproduced `all` request.

The shared `SectionOutcome` model is still the right mechanism. It already
defines safe `data`, `empty`, `degraded`, and `unavailable` states, drives JSON
and Markdown source-state rendering, and is used for analogous gene/protein
interaction failures. This ticket adds the missing drug-interaction adoption;
it does not create a pharmacogenomics-specific envelope or a second status
vocabulary.

## Test-first design

1. Add a pure drug-unit failure seam alongside `apply_approvals_result`: apply
   a `Result<DrugInteractionReport, BioMcpError>` plus an explicit label-source
   result that distinguishes successful nonblank text, successful empty text,
   and label-source failure, then complete the canonical `interactions`
   outcome. Do not use `Option<String>` alone for that input: it conflates a
   confirmed empty OpenFDA result with a failed OpenFDA request. Test this first
   with connection/API, timeout/source-unavailable, and malformed-response
   DDInter errors. Each must clear DDInter pagination/freshness/rows, expose only
   a bounded public message, never credit failed DDInter, and never serialize
   the private injected error detail. A surviving nonblank OpenFDA interaction
   text produces `degraded` and remains renderable; no surviving evidence
   produces `unavailable`. Also cover the full additive success/failure matrix:
   DDInter data plus a failed label source is `degraded` with the surviving
   DDInter/DrugBank contributors; DDInter empty plus a failed label source is
   `unavailable`; DDInter data plus a successful empty label is `data`; DDInter
   empty plus a successful empty label is `empty`; and successful label data
   with successful DDInter-empty is `data`. This prevents either source's
   failure from being mislabeled as a healthy empty.
2. Register drug `interactions` as a canonical source-backed state owned by
   additive `DDInter`, `DrugBank`, and `OpenFDA label` evidence. DrugBank is a
   contributor only when a successful structured DDInter row retains a
   nonblank matched DrugBank description; it is not credited merely because
   base resolution succeeded. Replace the drug-card interaction provenance's
   separately inferred entry with the canonical outcome projection so JSON
   contains exactly one `interactions` entry and
   `section_outcomes.interactions` and `_meta.section_sources` agree for data,
   empty, degraded, and unavailable. Keep the dedicated pageable interaction
   report's provenance behavior unchanged. Use the existing shared Markdown
   state renderer for the degraded/unavailable explanation; do not infer state
   from an empty interaction vector.
3. Change only multi-section aggregation (`all` or `interactions` plus at least
   one other distinct explicit section) to consume the interaction result
   through that seam and continue populating later sections. Repeated spelling
   of `interactions` does not create a partial-success request. A logically sole
   explicit `interactions` request must keep its current `source_unavailable`
   error and exit 1.
4. Add a deterministic process/spec fixture with a working local base drug and
   at least one working requested section plus an explicitly unavailable
   DDInter root. Cover both an OpenFDA label with interaction text and one
   without it. Prove Markdown prints the other surviving section and identifies
   interactions as degraded or unavailable, respectively. Prove JSON exits 0,
   remains one valid drug document, retains successful fields/outcomes, and
   reports the matching canonical interaction outcome and
   `_meta.section_sources` entry. Exercise the same JSON through typed MCP to
   prove transport does not replace the completed card with an error envelope.
   Put fixture setup in the existing runner/fixture scripts, not in executable
   Markdown.
5. Retain a process assertion for the sole `interactions` request: exit 1 and
   the existing structured error contract. This is the concrete all-failed case
   for this ticket and prevents partial-success handling from swallowing a
   request that returned no requested section.

## Scope

In scope: the drug interaction result/application seam, the canonical drug
interaction outcome and provenance entry, multi-section continuation after a
DDInter failure, Markdown/JSON/MCP parity, deterministic fixtures, and the drug
guide's source-section outcome list.

Out of scope: retries; timeout changes; DDInter synchronization or coverage
semantics; changing `all` to include approvals; changing single-section error
behavior; a new process exit code for partial results; and converting every
remaining legacy drug selector (`label`, `shortage`, or aggregate `regulatory`)
to a canonical outcome in this ticket. Those selectors have separate ownership
questions and are not needed to close the reproduced DDInter loss.

The observable generic guarantee is architectural rather than a repository-wide
rewrite: optional provider-backed interaction failures use the same
`SectionOutcome` contract already used by gene and protein, and a failure-state
matrix prevents a DDInter-only hardcoded exception.

## Acceptance

- With every non-DDInter branch served deterministically, `get drug <fixture>
  all` survives an unavailable DDInter root, renders the completed card, and
  exits 0; this directly guards the reported command rather than only a smaller
  explicit section list.
- With deterministic base/label interaction data and DDInter unavailable,
  `get drug <fixture> label interactions` prints the label interaction evidence,
  identifies the interactions section as degraded, and exits 0. The equivalent
  fixture without interaction text identifies it as unavailable.
- The JSON and typed MCP forms of both fixture cases return the drug card,
  retain the successful section, and agree on the canonical degraded or
  unavailable interaction outcome and provenance without leaking raw source
  errors.
- `get drug <fixture> interactions` alone retains the current error-only result
  and exit 1; repeating the same selector does not change that behavior.
- Successful interaction data and successful source-empty data retain their
  current biomedical fields and are typed `data` and `empty`, respectively;
  unit tests also distinguish a successful empty label lookup from a failed
  label lookup and preserve DrugBank credit when matched descriptions survive.
- Connection, timeout, and malformed-body failure tests all take the same
  generic result-application path.
- Focused Rust tests, the affected mustmatch page, `make lint`, `make test`, and
  `make spec` pass. No AlphaGenome behavior changes, so
  `make full-feature-check` is not required for this ticket.

## Dependencies

None. Ticket 1103 concerns recovery commands after a degraded/failed section;
it does not block preserving this card. Ticket 1099 concerns advertising valid
MCP section tokens and is also independent.

## Review

- Design review: REJECT (2026-09-04). The amended design correctly locates the
  early return and scopes partial success to aggregate drug gets, but it was not
  implementation-ready: (1) it registered only DDInter/OpenFDA even though
  successful interaction rows currently retain and credit matched DrugBank
  descriptions; (2) `Option<String>` could not distinguish a healthy-empty
  OpenFDA label result from label-source failure, so the additive outcome matrix
  could report false `data`/`empty` states; and (3) "two explicit sections" did
  not define duplicate-selector behavior, allowing a logically interaction-only
  request to evade the required exit-1 contract. The corrections above are
  required and need independent re-review before implementation.
- Independent design re-review: ACCEPT (2026-09-04). The corrected design is
  implementation-ready. The additive matrix now distinguishes successful-empty
  OpenFDA label lookup from label failure, makes `unavailable` truthful when no
  interaction evidence survives, and retains only the successful DDInter,
  DrugBank, and OpenFDA-label contributors in `degraded`/`data` states. DrugBank
  ownership matches the existing implementation: its description is credited
  only when a retained DDInter row carries a nonblank description from the
  resolved MyChem/DrugBank base. The canonical registry/provenance projection
  removes the current separately inferred duplicate while leaving the pageable
  report unchanged. The multi-section rule is decidable from normalized
  distinct selectors, preserves exit 1 for a logically sole `interactions`
  request, and does not conflict with typed MCP's existing duplicate rejection.
  Existing loopback provider fixtures, local bundle overrides, prepared MCP
  client, and runner-owned setup make the positive, degraded, unavailable, and
  error-only cases deterministic. Finally, the bounded static outcome messages
  and source-free `unavailable` shape prevent injected DDInter error details or
  failed-provider credit from crossing JSON, Markdown, or MCP trust boundaries.
- Code review: REJECT (2026-09-04). The canonical degraded outcome correctly
  credited only `OpenFDA label`, but the drug-card heading still inferred
  `Interactions (DDInter)` from the presence of label text. That presented the
  failed provider as if it owned the surviving payload. The review also found
  that the exact-`all` fixture had no local CIViC endpoint and asserted only
  targets plus interactions despite the stronger deterministic-branches claim.
- Remediation (2026-09-04): the drug-card heading now projects the canonical
  successful contributors and names DDInter only when the outcome credits it;
  the dedicated pageable report is unchanged. A failing-then-passing unit test
  and a process/spec negative assertion cover the OpenFDA-label-only degraded
  card. The section-outcome fixture now serves an explicit local empty CIViC
  GraphQL response, and the exact-`all` assertion checks the surviving label,
  approvals/regulatory compatibility array, shortage array, safety, targets,
  indications, CIViC, and interactions.
- Code re-review: REJECT (2026-09-04). The product correction was sound, but
  the pre-existing cross-cutting Markdown fixture constructed a structured
  DDInter row with a DrugBank description while leaving the new canonical
  `section_outcomes.interactions` state as `not_requested`. Its expected
  `Interactions (DDInter)` heading therefore no longer described its own typed
  fixture state.
- Second remediation (2026-09-04): the cross-cutting fixture now completes the
  intended successful interaction outcome with `DDInter` and `DrugBank`, matching
  its structured row and nonblank matched description. No payload-based heading
  inference or product behavior was added.
- Primary gate discovery (2026-09-04): `make lint` passed its license,
  advisory, formatting, and ordinary lint checks but the Rust source-size
  ratchet rejected `src/entities/drug/get.rs` at 1,187 lines against its exact
  1,097-line baseline. The same audit required the touched
  `src/render/provenance.rs`, then 1,817 lines, to match its pinned 1,823-line
  baseline rather than drift below it while remaining over the 1,000-line
  threshold.
- Source-size remediation (2026-09-04): label interaction classification,
  result application, additive outcome construction, and async card interaction
  orchestration moved into `src/entities/drug/interactions.rs`, beside the
  report types and DDInter aggregation they own. Internal report/application
  helpers are private; only the card seam and test seam have parent-module
  visibility. `get.rs` is exactly 1,097 lines, `provenance.rs` is exactly 1,823
  lines, the new owner is 423 lines, and the unchanged inventory now passes
  with zero findings across 692 tracked Rust files. The complete `make lint`
  gate then passed, including formatting, Clippy with warnings denied,
  license/advisory checks, and the quality ratchets.
- Primary test-gate discovery (2026-09-04): after 1,279 passing tests,
  `make test` found one stale explicit drug registry-key contract. The runtime
  correctly initialized canonical `interactions`, but
  `omitted_drug_outcomes_initialize_every_registry_key` still expected only the
  five pre-ticket keys.
- Registry-key remediation (2026-09-04): the explicit ordered expectation now
  includes `interactions` between `indications` and `safety`, matching the
  registry-backed `BTreeMap` order. The dynamic registry construction and key
  validation remain unchanged; neighboring searches found no second exact drug
  key inventory.
- Final code re-review: ACCEPT (2026-09-04). The reviewer verified the canonical
  provider projection, deterministic local CIViC-backed `all` fixture, migrated
  successful-DDInter and registry-key contracts, extracted interaction module,
  unchanged pageable path, bounded error handling, and exact source-size
  baselines. No blocking or optional findings remain.

## Implementation evidence

Implemented on 2026-09-04. The drug aggregation path now classifies normalized
distinct selectors, applies DDInter and OpenFDA-label results through one
additive interaction seam, and continues only for `all` or a true
multi-section request. A sole `interactions` request and duplicate-only spelling
still return the original hard error. Drug interaction state is canonical in
the registry and `_meta.section_sources`; the former inferred card provenance
entry was removed. DDInter pagination, freshness, and rows are cleared on
failure, while surviving label text remains available under a bounded
`degraded` outcome. No raw provider error is retained in the card.

Test-first red evidence: the initial focused Rust test did not compile because
`allow_partial_interactions`, `LabelInteractionResult`, and
`apply_interactions_result` did not exist. After implementation:

- drug get tests: 23 passed;
- drug provenance tests: 4 passed;
- drug Markdown tests: 18 passed;
- source-state registry quality-ratchet test: 1 passed;
- affected `spec/entity/section-outcomes.md`: 12 passed, including deterministic
  Markdown, JSON, typed-MCP, original `all`, sole-interaction, and
  duplicate-interaction coverage;
- `cargo fmt --all -- --check`, `git diff --check`, and
  `cargo clippy --locked --no-default-features --lib --tests -- -D warnings`
  passed.

The architecture inventory required the same canonical drug-interaction row as
the runtime registry; adding it was a documentation synchronization, not a
material design correction. `make lint` passed after the source-size
remediation; continuation of `make test` and `make spec` remains with the
primary agent's final gates.

Code-review remediation added one learned presentation constraint: provider
names in a drug-card heading are provenance claims and must therefore come from
the canonical successful-contributor list, not from payload presence. The new
heading regression was red before that correction and green afterward. The
affected executable spec remains 12 passing cases with the locally served CIViC
branch and expanded exact-`all` assertion.

The primary lint gate also enforced the intended module boundary: optional
interaction source-state policy belongs with drug interaction aggregation, not
in the already pinned general drug-get orchestrator. The extraction preserved
the reviewed selector classification and canonical heading behavior without a
source-size baseline change.

## Completed 2026-09-04

All acceptance criteria are implemented. Multi-section drug cards now preserve
working sections when DDInter is unavailable, report canonical degraded or
unavailable interaction state with truthful successful-source attribution, and
retain the existing hard failure for a logically sole interactions request.

Final primary gates passed on the independently accepted tree: `make lint`;
`make test`, including the complete offline Rust lane, 883 Python tests passed
and 3 skipped, and the strict documentation build; and `make spec`, including
the 12-case section-outcomes page, fixture cleanup, 38 isolation contracts, and
the 8-case static lane.
