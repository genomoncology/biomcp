---
flow: build
priority: 2
---

# Label FAERS percentages as report shares, not incidence

## Outcome

Every FAERS percentage shown by an adverse-event search identifies its actual
denominator and explicitly says that it is not an incidence rate or evidence
of causality. CLI JSON and the raw `biomcp` MCP escape hatch carry the same
meaning where those surfaces contain reaction percentages. Counts and
arithmetic do not change.

## Current facts and root cause

At `27933c96`, `templates/adverse_event_search.md.j2` renders a generic
`Percent` column. It precedes the table with one of these arithmetic notes:

```text
Top reactions: aggregate count query; Percent = count / total reports.
Top reactions: returned-report sample only; Percent = count / returned reports, not population frequency.
```

The first note is used by `biomcp drug adverse-events <name>` after it combines
an OpenFDA `count=patient.reaction.reactionmeddrapt.exact` response with the
total number of records matching the same filters. The second is used by
`biomcp search adverse-event`: `summarize_search_results` counts each reaction
at most once in each returned report and divides by the number of reports on
that returned page. Thus the values are, respectively:

- the percentage of all matching FAERS report records in which a reaction term
  occurs; or
- the percentage of returned-page FAERS report records in which a reaction
  term occurs.

Neither denominator is the number of people treated or exposed. FDA describes
FAERS as spontaneous safety reports, says reporting is incomplete and affected
by reporting behavior, and says these records cannot estimate incidence or
establish cause and effect. OpenFDA also explains that one report can contain
multiple products and reactions without connecting an individual product to an
individual reaction:

- <https://open.fda.gov/apis/drug/event/>
- <https://open.fda.gov/apis/query-syntax/>
- <https://www.fda.gov/drugs/cder-conversations/understanding-cders-postmarket-safety-surveillance-programs-and-public-data>

The numeric `percentage` in JSON has no semantic sibling, so CLI JSON and raw
MCP callers cannot distinguish the aggregate denominator from the returned-page
denominator without knowing which command produced it. This is a framing
defect in both human and structured output, not a provider-query defect.

The original ticket's separate co-reported-drug claim is no longer a current
gap. `transform::adverse_event::suspect_drug_names` permits only
`drugcharacterization == "1"` in search-row `drug`; report detail separately
maps characterization `"2"` products into `concomitant_medications`, rendered
under `Concomitant Drugs`. Do not change this established behavior. FAERS still
does not link a report's individual reactions to any one listed drug, which is
why the causality caveat remains necessary.

## Required behavior

### Markdown

- Replace the generic `Percent` heading with denominator-specific language:
  `Share of matching reports` for the aggregate path and `Share of returned
  reports` for the returned-page sample path.
- Immediately above the reaction table, explain the numerator and denominator
  in plain English. The sample path must also remain explicit that the returned
  page is only a sample of all matching reports.
- In both paths, print one short, prominent caveat containing the exact concepts
  `not incidence` and `does not establish causality`. It must say that the
  denominator is FAERS reports, not treated/exposed patients; arithmetic alone
  is insufficient.
- Keep reaction terms, counts, percentages, rounding, ordering, report rows,
  pagination, and source provenance unchanged.
- `search adverse-event --count` already labels its derived column `Percent of
  Shown Count`; add a parallel not-incidence/not-causality caveat because it is
  still a transformation of spontaneous-report counts. This path's denominator
  is the sum of the shown bucket counts, not the number of distinct FAERS
  reports: one report can contribute to multiple buckets. State that boundary
  rather than reusing language that calls its denominator a report count. Its
  computation and JSON bucket-only contract remain unchanged; because that JSON
  contains counts but no percentages, it does not gain `percentage_context`.

### JSON and raw MCP

- Add one compact `percentage_context` object to
  `AdverseEventSearchSummary` whenever reaction percentages are present. It
  has this stable shape:

  ```json
  {
    "measure": "share_of_faers_reports",
    "denominator": "all_matching_reports",
    "denominator_count": 20,
    "is_incidence": false,
    "establishes_causality": false
  }
  ```

  The sample-path denominator value is `returned_reports`; the aggregate-path
  value is `all_matching_reports`. `denominator_count` is exactly the divisor
  used for the displayed percentages (`returned_report_count` or
  `total_reports`). Omit `percentage_context` when no reaction percentage rows
  exist rather than emitting a fictitious denominator.
- Keep existing `summary.total_reports`, `summary.returned_report_count`, each
  row's `reaction`, `count`, and `percentage`, and all surrounding envelopes
  unchanged. The context object is additive and must tolerate deserializing an
  older summary that lacks it.
- The raw MCP `biomcp` tool runs this existing CLI search surface and, with
  `json: true`, returns its CLI JSON text. It must expose the identical context
  object and enum strings. Raw MCP Markdown must contain the same labels and
  caveat as ordinary CLI Markdown; do not add an MCP-only representation.
- The typed MCP `search` tool does not publish an `adverse-event` branch. Its
  deliberate first slice currently covers author, gene, PGx, GWAS, article,
  trial, variant, and protein; typed `get adverse-event` is a report-detail
  surface and never returns this search summary. Do not add a new typed search
  entity in this framing ticket. That would require designing the entire
  adverse-event filter/source/type union and re-budgeting a nearly saturated MCP
  catalog, not merely exposing `percentage_context`.

Use a Rust denominator enum (serialized in snake case) and one owning summary
constructor/helper. Do not infer semantics in Jinja from the human
`summary_source_label`; that label currently decides the branch by substring
matching `"aggregate"` and is not a trustworthy data contract.

## Test-first acceptance

1. Before production edits, add focused failing summary tests for both paths.
   With stable synthetic rows, prove the existing counts and numeric
   percentages are unchanged while the aggregate context is
   `all_matching_reports` with `denominator_count == total_reports`, and the
   sample context is `returned_reports` with `denominator_count ==
   returned_report_count`. Prove a summary with no reaction rows omits the
   context and old JSON without the additive field still deserializes.
2. Add renderer tests for both paths that assert the exact denominator-specific
   column heading and an adjacent caveat containing `not incidence`, `treated`
   or `exposed`, and `does not establish causality`. Assert the misleading bare
   `| Reaction | Count | Percent |` header is absent and the existing numeric
   cells remain byte-for-byte unchanged.
3. Extend the count-renderer test to prove `Percent of Shown Count` and its
   arithmetic remain unchanged while the FAERS caveat is visible. Prove that
   the accompanying text identifies the sum of shown bucket counts as the
   denominator and does not mislabel it as a distinct-report denominator.
4. Extend an existing fixture-backed CLI/process contract to prove sample JSON
   serializes `percentage_context` exactly and Markdown has the same meaning.
   Extend a real-transport MCP contract to run `search adverse-event` through
   the raw `biomcp` tool in both JSON and Markdown modes and prove it preserves
   those same semantics. Do not claim coverage through typed MCP `search` or
   typed `get`; neither is this search-summary surface. No public network is
   required.
5. Add an existing `spec/entity/drug.md` block, using the provider-contract
   fixture and runner, that proves `drug adverse-events pembrolizumab` labels
   the aggregate percentage as a share of matching reports, says it is not
   incidence or causality, and retains the fixture's count/percentage. The
   current fixture combines an event page whose metadata says one matching
   report with a captured count bucket of 12,016 records; that impossible
   relationship would produce a 1,201,600% "share." Make those two existing
   fixture responses arithmetically coherent before treating the spec as
   semantic evidence, without changing runtime arithmetic or adding a file.
   Prove the JSON aggregate context with `jq`; specification prose must not
   invoke a Cargo command.
6. Update the adverse-event user guide, the OpenFDA source guide, and the
   adverse-event summary reference so humans and agents see the two denominator
   meanings and the FDA limitations. Do not rewrite historical blog examples.
7. Run the focused adverse-event entity, renderer, CLI/process, and MCP tests,
   then run exactly:

   ```bash
   make lint
   make test
   make spec
   ```

   Finally run `git diff --check` and verify the package list is exactly 1300.

## Scope, rails, and non-goals

Likely existing files are:

- `src/entities/adverse_event.rs`
- `src/render/markdown/adverse_event.rs`
- `src/render/markdown/adverse_event/tests.rs`
- `templates/adverse_event_search.md.j2`
- `tests/adverse_event_route_contract.rs`
- `tests/rmcp_client_contract.rs`
- `spec/entity/drug.md`
- `docs/user-guide/adverse-event.md`
- `docs/reference/drug-approvals-and-ae-summary.md`
- `docs/sources/openfda.md`

The package inventory is already exactly 1300 files; use existing files only.
The global Rust-source threshold is 1000 lines. At design time the relevant
smaller files are 379 lines (`render/markdown/adverse_event.rs`), 401 lines
(`render/markdown/adverse_event/tests.rs`), 391 lines
(`tests/adverse_event_route_contract.rs`), and 697 lines
(`tests/rmcp_client_contract.rs`). `src/entities/adverse_event.rs` is pinned at
exactly 2663 lines under the quality-ratchet inventory: any edit there must
remain net-zero lines (or perform a valid decomposition into existing files),
and the inventory must not be raised.

- No OpenFDA query, filter, count, pagination, ranking, deduplication, rounding,
  or source-selection change.
- No incidence calculation, exposed-patient denominator, disproportionality
  statistic, comparison between products, causal inference, or clinical claim.
- No changes to VAERS or trial-reported adverse-event percentages; their
  providers and denominator contracts differ.
- No new typed MCP `search` entity and no change to typed MCP `get`; the raw
  MCP escape hatch already exposes the affected CLI commands.
- No restructuring of suspect versus concomitant drugs and no attempt to link
  a reaction to one drug within a multi-drug report.
- No removal or rename of existing JSON keys. `percentage_context` is additive.
- No prerequisite ticket or live-network acceptance test is required.

## Design recommendation

**ACCEPT after independent design review.** The misleading Markdown header
remains live, and the structured summary lacks its denominator semantics. The
present code proves both reaction-summary arithmetic paths, while authoritative
FDA/OpenFDA material supports the non-incidence and non-causality boundary. The
review corrected three test/design hazards: generic count output uses the sum
of shown bucket counts rather than a distinct-report denominator; the shared
provider fixture's event total must be made coherent with its 12,016-record
reaction bucket before an executable spec can honestly call the result a
percentage share; and adverse-event search is available over raw MCP but is not
a published typed-search entity. Adding that substantial new typed filter
surface is not required to repair the existing percentage contract. The stale
co-reported-row claim remains outside scope because the current transform
already separates suspect and concomitant drugs.

## Implementation evidence

- Red: the focused summary contract failed on the missing typed denominator and
  `percentage_context` field (`E0432`/`E0609`) before production edits.
- Green: summary compatibility/divisor 1/1, adverse-event Markdown 14/14,
  adverse-event entity 34/34, CLI process 8/8, and raw stdio MCP 1/1 passed.
- Repository gates: `make lint` passed; `make test` passed 3,147 Rust tests
  (30 skipped), 892 Python contracts (3 skipped), and strict docs; `make spec`
  passed after correcting the new fixture assertion from `100%` to the
  unchanged one-decimal rendering `100.0%`.
- Rails: the package remains exactly 1,300 files, the pinned adverse-event
  entity remains exactly 2,663 lines, every changed smaller Rust file remains
  below 1,000 lines, the capture-receipt audit passed, and `git diff --check`
  passed.

## Independent code review

**ACCEPT with no findings.** Review confirmed both denominator formulas,
case-insensitive per-report reaction deduplication, additive optional
snake-case context and legacy deserialization, denominator-specific Markdown
and safety caveats, unchanged generic count JSON and arithmetic, raw stdio MCP
parity, unchanged queries and suspect/concomitant handling, coherent synthetic
fixture classification, documentation/spec accuracy, and all package and
source-size rails. Seven targeted contracts, 49 broader entity/renderer tests,
the suspect-drug regression, capture-receipt audit, and `git diff --check`
passed independently.

## Completed 2026-09-05

FAERS reaction summaries now identify their actual denominator in Markdown and
additive JSON context: all matching reports for aggregate summaries and
returned-page reports for sampled summaries. Both paths explicitly state that
report shares are not incidence in treated or exposed patients and do not
establish causality. Generic count output accurately describes its shown-bucket
denominator without changing its JSON shape. CLI and raw MCP surfaces agree,
and the synthetic provider fixture is arithmetically coherent and truthfully
classified.

Primary verification passed after independent review: `make lint`; `make test`
(3,147 Rust tests passed with 30 skipped, 892 Python tests passed with 3
skipped, and strict documentation passed); and `make spec` (all routine groups,
including 140 serialized cases with 4 skipped, 39 parallel-isolation cases,
and 8 static cases). Packaging remains exactly 1,300 files, the pinned
adverse-event entity remains exactly 2,663 lines, and `git diff --check`
passes.
