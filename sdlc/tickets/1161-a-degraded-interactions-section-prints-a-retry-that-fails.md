---
flow: build
priority: 8
---

# A degraded interactions section prints a retry command that fails

## Goal

`get drug <identity> interactions` renders the existing typed interaction
outcome whenever base drug identity resolves. The canonical recovery command
then remains executable without inventing an unrelated second section or
discarding surviving evidence.

Observed at commit `f68d8832` with release binary
`0.9.0-dev.6+gf68d8832` on 2026-09-05, using an empty directory as
`BIOMCP_DDINTER_DIR` to make DDInter unavailable:

```text
$ biomcp get drug imatinib label interactions
## Interactions

**Interactions status (DDInter / DrugBank / OpenFDA label):** degraded (partial/incomplete) — Drug interaction evidence is incomplete because a source was unavailable.
Retry: `biomcp get drug "imatinib mesylate" interactions`
## Additive Label Text (OpenFDA)
...

$ biomcp get drug "imatinib mesylate" interactions
Error: Source unavailable: DDInter is not available. Review source configuration and retry.
$ echo $?
1
```

The same `Retry:` line is printed from `biomcp get drug imatinib all`. The
JSON form of the printed command returns the `source_unavailable` error
envelope instead of a card, also with exit 1. Gene recovery commands already
have the intended behavior: forced failures for `pathways`, `ontology`,
`diseases`, `interactions`, and `expression` render their typed section state
and exit 0.

## Why the two contracts collide

Record 1098 requires that a logically sole explicit `interactions` request
keep its `source_unavailable` error and exit 1, and lists retries as out of
scope. Record 1103, landed later the same day, prints a sole-section retry for
every degraded or unavailable registered section. Neither record cites the
other. The recovery affordance therefore points at the one request shape that
is contractually required to refuse.

## Contract settlement

Reverse the narrow record 1098 boundary for the `get drug` card only.
`populate_card_interactions` already feeds the DDInter result and the
presence-aware `LabelInteractionResult` through `apply_interactions_result`.
A logically sole selector must pass the DDInter failure to that reducer instead
of propagating it. Duplicate selectors are idempotent: `interactions
interactions` has exactly the same provider work and settlement as one
selector.

Do not change the retry to `label interactions`, `all`, or an arbitrary second
section. `resolve_drug_base` deliberately uses `label_required = false` for a
sole `interactions` request, so OpenFDA construction, transport, HTTP, or decode
failure becomes `label_attempt_failed` while a resolved MyChem base card
survives. Explicit `label interactions` and `all` include the required label
section; the same OpenFDA failure must still abort before DDInter work. A
second selector such as `targets interactions` must not be needed to enable
partial interaction settlement.

The separate `biomcp drug interactions <name>` pageable report remains a
different product surface. It may omit label text when OpenFDA fails, as it
does today, but DDInter is its required data owner: DDInter readiness or read
failure remains a command error with exit 1. This ticket does not give that
report `section_outcomes` or convert its DDInter failure into a card.

## Frozen interaction outcome matrix

The reducer remains the single truth owner. Its public outcome and contributor
list are:

| DDInter result | OpenFDA label result | Outcome | `sources` |
| --- | --- | --- | --- |
| rows with no DrugBank narrative | failed | `degraded` | `DDInter` |
| rows with a DrugBank narrative | failed | `degraded` | `DDInter`, `DrugBank` |
| healthy empty | failed | `unavailable` | empty |
| failed | label interaction text | `degraded` | `OpenFDA label` |
| failed | healthy empty | `unavailable` | empty |
| failed | failed | `unavailable` | empty |
| rows with no DrugBank narrative | healthy empty | `data` | `DDInter` |
| rows with a DrugBank narrative | healthy empty | `data` | `DDInter`, `DrugBank` |
| rows with no DrugBank narrative | label interaction text | `data` | `DDInter`, `OpenFDA label` |
| rows with a DrugBank narrative | label interaction text | `data` | `DDInter`, `DrugBank`, `OpenFDA label` |
| healthy empty | label interaction text | `data` | `OpenFDA label` |
| healthy empty | healthy empty | `empty` | `DDInter`, `OpenFDA label` |

When both sources contain evidence, `data` contains their ordered union,
including `DrugBank` only when a retained DDInter row has a non-empty legacy
DrugBank description. A failed provider is never credited. `degraded` and
`unavailable` use the existing bounded public messages; provider error bodies,
paths, URLs, and transport details never enter the entity, Markdown, JSON, or
MCP response.

## Test-first acceptance

1. First change the focused parser/settlement tests so a sole and a repeated
   `interactions` selector permit typed partial settlement. Retain an explicit
   table-driven reducer test for every row above, including the conditional
   `DrugBank` contributor, exact source order, payload clearing, and bounded
   non-leaking messages. Then make the smallest runtime change.
2. Extend the existing section-outcomes fixture, not a new fixture family, with
   deterministic identities for: DDInter failure plus label evidence; DDInter
   evidence with and without a DrugBank narrative plus OpenFDA label
   acquisition failure; healthy-empty DDInter plus label failure; and both
   sources failing. Record exact request counts/order where needed to prove the
   required-label exclusions abort before DDInter rather than merely producing
   the expected text.
3. For each new failure settlement, cover ordinary CLI Markdown and CLI JSON.
   Markdown asserts the exact status label, public message, contributor label,
   retained evidence, exactly one adjacent `Retry:` command, and no upstream
   failure detail. JSON asserts exit 0, the exact
   `section_outcomes.interactions` object, the one agreeing
   `_meta.section_sources` row, and exactly one equal command in
   `_meta.next_commands`.
4. Exercise the same requests through both MCP entry points: raw-shell
   `biomcp` and typed `get`. For each entry point, cover default Markdown and
   `json: true`. The Markdown result has the same status, attribution,
   evidence, and exact retry as CLI Markdown. Structured results have the same
   entity outcome, provenance row, and next command as CLI JSON. Successful
   cards are non-error MCP results; neither output mode contains the injected
   failure sentinel.
5. Extract the retry from CLI Markdown and compare it byte-for-byte with the
   matching `_meta.next_commands` member. Execute both representations under
   the same injected DDInter/OpenFDA condition that produced them. They must
   parse as the canonical sole `get drug <resolved-name> interactions`
   request, return exit 0/non-error MCP results, and reproduce the exact typed
   outcome and sources. Do not validate only against a subsequently healthy
   fixture. Repeat this proof for an unavailable result as well as a degraded
   result.
6. Freeze the exclusions explicitly. With DDInter unavailable, the separate
   `biomcp drug interactions <name>` Markdown and JSON commands still return a
   command error/`source_unavailable` envelope and exit 1. With base identity
   available but OpenFDA label acquisition failing, the composite
   `get drug <name> label interactions` request and `get drug <name> all` both
   fail before any DDInter read; assert the OpenFDA error, exit 1, and a zero
   DDInter request/read count.
7. Keep the central registry/recovery invariants in
   `src/entities/source_state_registry.rs`, the printed-command round-trip in
   `src/cli/tests/printed_card_commands.rs`, the detail Markdown/JSON agreement
   in `src/cli/tests/surface_agreement.rs`, and the MCP read-only allowlist
   assertion in the existing `src/mcp/shell.rs` test module green. Add focused
   drug assertions only where an existing test lacks this sole-selector
   behavior; do not duplicate record 1103's exhaustive generic recovery
   coverage. Keep one named healthy gene recovery case and one unrelated drug
   section case as bounded regressions instead of claiming every route through
   an unbounded integration matrix.
8. Update the executable contract in `spec/entity/section-outcomes.md`, its
   existing `spec/fixtures/setup-section-outcomes-spec-fixture.sh` setup, the
   existing `spec/fixtures/run-section-outcome-mcp.sh` runner, and the existing
   `section-outcome-interactions` mode in
   `examples/rmcp_streamable_http_contract.rs`. Replace the obsolete sole-
   interaction hard-failure examples; retain the multi-section and healthy-
   empty contracts. No live network is required.

## Dependencies and boundaries

Ticket 1151 depends on this ticket (`1151` declares `deps: [1161]`) because it
will expand structured drug-section discovery after the recovery target is
made executable. This ticket must not depend on 1151, add 1151's undisclosed
section commands, or change their ordering. There is no reverse dependency or
typed-MCP schema change here: the existing typed `get` projection already
accepts the `interactions` section.

This ticket changes only the sole-section outcome contract for the `get drug`
card and its existing recovery affordance. It does not change the separate
pageable report, DDInter coverage or synchronization, source timeouts, retry
middleware, identity matching, any other section's failure classification, or
any recovery route.

## Implementation surfaces and repository rails

- Runtime ownership stays in `src/entities/drug/interactions.rs` and the
  existing section parsing/plumbing in `src/entities/drug/get.rs`; colocated
  tests own the complete reducer and duplicate-selector matrix.
- Rendering and structured projection reuse the existing drug renderer,
  section recovery registry, and provenance projection. Do not add template-
  specific command construction or a second outcome reducer.
- Extend the existing `spec/entity/section-outcomes.md`, section-outcomes
  fixture scripts, MCP contract-example mode, and existing test modules. Do
  not add a tracked file or dependency: the source package is already exactly
  1300 files.
- `src/entities/drug/get.rs` is pinned at exactly 1094 lines and must not grow.
  `src/mcp/shell.rs` is pinned at exactly 2136 lines and must not grow. Never
  increase any other pinned over-limit inventory; keep ordinary Rust sources
  at or below 1000 lines and `src/cli` Rust files at or below 700 lines.
- Run the focused Rust tests and the focused Python/spec contracts first, then
  `make lint`, `make test`, and `make spec`. Finish with
  `cargo package --list --allow-dirty --locked --offline --no-verify | wc -l`
  equal to 1300 and `git diff --check`. The completion record must distinguish
  focused evidence from full gates and state the actual results.

## Design review

Accepted on 2026-09-06 after an independent rereview against `201274c0`.
The reviewer confirmed that the frozen matrix matches the current reducer,
that the sole-card and pageable-report boundaries are explicit, and that the
retry, CLI, raw-MCP, and typed-MCP proofs are implementable. The reviewer also
confirmed that intervening changes on main do not alter the relevant drug,
registry, fixture, or MCP surfaces.

## Implementation evidence

Implemented test-first on 2026-09-06 against `6ed5b39b`. The sole and repeated
interaction selectors now enter the existing reducer instead of propagating a
DDInter error. The frozen twelve-row contributor matrix, all twenty-three
focused drug-get tests, MCP example compilation, and the initial fourteen-case
`section-outcomes.md` executable contract pass. The contract exercises CLI and
both MCP entry points in Markdown and JSON, executes degraded and unavailable
recovery commands under the same failures, retains the pageable-report error,
and covers DDInter evidence when OpenFDA label acquisition fails. Formatting,
shell syntax, diff whitespace, and the pinned source-file ceilings pass.

The first independent code review rejected incomplete executable coverage and
an overclaimed 1,097-line source baseline. Remediation restores the truthful
1,094-line exact baseline and adds the missing failure-surface and exclusion
proofs. The expanded fifteen-case contract passes, as do the focused MCP
allowlist, printed-card recovery, surface-agreement, source-state-registry, and
six package-boundary checks. The quality ratchet, formatting, diff whitespace,
fixture shell syntax, exact 1,300-file package inventory, and the 1,094/2,136
Rust source ceilings also pass. Full repository gates were deliberately not
rerun during this focused remediation. Focused re-review is pending, so this
ticket remains in `sdlc/tickets/` and is not yet a completion record.
