---
flow: build
priority: 6
deps: ["0957"]
base: edca929f
head: 988844b4
---
# Make provider health and section states automation-safe

Health cannot target one provider or opt into a failing exit, while some human
section renderers say both "no results" and "provider unavailable." Give
operators a precise selection/exit policy and make every renderer respect the
typed source outcome it already receives.

## Health contract

Add repeatable `--api <canonical-name>` filters and `--fail-on-error`.
Canonical names are the exact case-insensitive names emitted by the typed health
catalog; unknown or ambiguous names fail before any probe and show bounded
canonical suggestions. Repeated values form a deduplicated union. Only selected
providers are constructed or contacted, while existing parallelism remains for
more than one selection.

Supplying any `--api` implies the existing API-only view: the eight local data,
filesystem, and cache readiness rows are neither checked nor rendered.
Combining it with `--apis-only` is accepted as a redundant no-op. With no
`--api`, both the ordinary full report and explicit `--apis-only` retain their
current row sets.

Default exit behavior stays compatible. With `--fail-on-error`, finish and
render the full selected report, then exit 1 exactly when the existing
`HealthReport.error` count is nonzero. Rows classified by the current probe
model as warning or excluded, including `not_configured`, do not fail. Do not
invent a new health status: the row continues to use the current `HealthStatus`
values, including `error` and `unavailable`, while the existing `ProbeClass`
decides the summary bucket. In JSON mode this is one valid typed health report
on stdout with `exit_policy`, `ok`, and the existing exact
healthy/warning/excluded/error counts, with no stderr prose. This is a completed
health report whose requested policy failed, not a transport-error envelope.

## Section-state contract

One shared renderer mapping handles the current six `SectionOutcomeState`
values without changing the schema: `not_requested` renders no absence claim;
`inapplicable` explains why the section does not apply; `data` renders data;
successful `empty` says the healthy source returned no results; `degraded`
labels partial/incomplete data; and `unavailable` says no conclusion can be
drawn. No state prints a contradictory second state. JSON typed outcomes remain
the authority; Markdown cannot infer absence from an empty value alone.

## Done when

- A 54-provider fixture proves one `--api` makes exactly one probe; repeated,
  mixed, unknown, and ambiguous selectors are deterministic and pre-network.
- Process tests cover every selected-state combination, default versus failing
  exit policy, JSON validity on exit 1, and unchanged unfiltered behavior.
- Shared renderer fixtures cover all six section states, including the confirmed
  DisGeNET and PharmGKB contradictions, and a ratchet enumerates every renderer
  using a provider-backed optional section.
- CLI/MCP schemas and health/operator docs define names and exit semantics.

## Authorized test changes

Design commits may restate health arguments/catalog selection, typed summaries,
exit plumbing, shared source-outcome rendering, affected section renderers,
fixtures, schemas, and docs. Provider-specific biomedical interpretation is out
of scope.

The src line ceiling may rise by at most 220 lines.

## Completion

Health accepts repeatable exact case-insensitive provider filters, rejects bad
names before client construction, and offers report-first `--fail-on-error`
semantics for automation. JSON exposes the exit policy and overall result. A
shared six-state human mapping now keeps not-requested, empty, degraded, and
unavailable outcomes distinct, so absence claims are made only after a healthy
empty result and partial or unavailable sources cannot imply negative evidence.
