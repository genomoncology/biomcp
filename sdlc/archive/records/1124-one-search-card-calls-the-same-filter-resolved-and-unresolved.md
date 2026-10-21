---
flow: build
priority: 2
---

# Give variant identity and filter evaluation distinct vocabularies

Status: proposed

## Outcome

Variant-search cards use `resolved` / `ambiguous` / `unresolved` only for an
exact requested variant identity. Per-filter output instead says whether each
submitted filter was `evaluated` or `unavailable`, in both Markdown and JSON,
so a true empty result cannot read as a contradiction.

## Current facts

The live command still reproduces on 2026-09-05 from clean `ab61d6e8`
(`biomcp 0.9.0-dev.6`):

```text
$ biomcp search variant -g RB1 --hgvsp Q999X --limit 1
Requested variant: RB1 Q999X
Resolution: Unresolved
...
## Filter resolution
gene: resolved
hgvsp: resolved
```

The matching JSON carries `resolution.status: "unresolved"` beside
`filter_resolution.hgvsp: "resolved"`. The live provider still returns the
positive control `H3F3A K28M` as one resolved identity, and gene-only `RB1`
still has no exact-identity block.

These signals answer different questions and may legitimately differ. Exact
identity resolution is computed after provider candidates are compared with
`requested_variant`: one compatible identity is resolved, no compatible
identity after an exhaustive scan is unresolved, and incomplete or multiple
evidence is ambiguous. The filter map says whether BioMCP could evaluate each
accepted filter well enough for an empty intersection to be informative. For
example, the same live build reports an invented gene as unresolved while
still reporting its syntactically valid `hgvsp` filter as resolved. Distinct
per-filter outcomes are useful; reusing the identity words is not.

The collision is introduced in two presentation paths from one page model:

- `src/entities/variant/search/mod.rs` builds `filter_resolution` as a
  `BTreeMap`; all accepted filters default to `Resolved`, while the existing
  `GeneUnavailable` diagnostic overrides only `gene`.
- `src/cli/variant/dispatch.rs` independently prefixes the exact-identity
  `Resolution: ...` line, then passes that map to
  `templates/variant_search.md.j2` and `src/render/json.rs`.
- CLI Markdown and JSON, raw MCP, and typed MCP `search` all traverse this same
  dispatch and renderer path. There is no separate MCP response model to fix.

Ticket 1094 intentionally established that `RB1 + Q999X` is an evaluated query
with no matching identity, not an unavailable filter. This ticket must preserve
that negative-evidence distinction rather than relabel `hgvsp` as unresolved.

## Required behavior

Use one typed filter-evaluation map as the source for every output surface:

- Replace the public `filter_resolution` signal with `filter_evaluation`.
- Its closed statuses are `evaluated` and `unavailable`. `evaluated` means the
  accepted filter could participate in an interpretable provider query; it
  does not claim that the value names a record or that the requested variant
  exists. `unavailable` preserves the existing `GeneUnavailable` case, where a
  zero result is not reliable negative evidence for that filter.
- Markdown labels the exact block `Variant identity: <status>` and the map
  `## Filter evaluation`. JSON retains the established `requested_variant` and
  `resolution` object, and emits the same map once as `filter_evaluation`.
- Do not retain `filter_resolution` as a second JSON field or Markdown block.
  A duplicate compatibility alias would restore two sources of truth. This is
  a correction to a field introduced in the unreleased `0.9.0-dev.6` surface.

The map remains a non-null object with exactly one canonical entry per
submitted filter, in deterministic `BTreeMap` key order. Existing canonical
names, including `residue_alias`, stay unchanged. Mixed per-filter states are
valid: one input may be unavailable while another was evaluated. Exact
`requested_variant` and `resolution` remain present together for strict
identity searches and absent together for broad searches.

## Scope

- Rename the filter map/type/status vocabulary at its owning variant-search
  model and thread that same value through the shared JSON and Markdown
  renderers.
- Update the existing deterministic variant spec, renderer/envelope tests, and
  variant-search documentation to define both questions explicitly.
- Preserve provider requests, alias retry and diagnostics, candidate identity
  comparison, rows, ranking, counts, pagination, and result-table contents.
- Do not add filter probes, infer matches from result count, or special-case
  `RB1`, `Q999X`, or any fixture value.
- Do not change variant detail or variant-article resolution contracts.

## Acceptance

Test first against the existing deterministic variant identity fixture:

1. `RB1 + Q999X` renders `Variant identity: unresolved`, then `gene: evaluated`
   and `hgvsp: evaluated` under `Filter evaluation`; neither Markdown nor JSON
   contains the retired `filter_resolution` signal.
2. Its JSON keeps the exact `resolution.status == "unresolved"` and exposes
   `filter_evaluation == {gene:"evaluated", hgvsp:"evaluated"}`.
3. `H3F3A + K28M` remains one resolved identity with both filters evaluated.
   Gene-only search omits exact-identity metadata while retaining its filter
   evaluation.
4. A generic synthetic test covers an unavailable gene beside an evaluated
   second filter, one canonical key per filter, lexicographic rendering order,
   and the non-null empty/default map without relying on a named live record.
5. Raw and typed MCP parity is proved at their shared CLI execution seam for
   both Markdown and JSON; no MCP-only vocabulary or response model is added.
6. Existing assertions continue to prove unchanged requests, results, counts,
   pagination, diagnostics, and positive exact resolution.

Run `make lint`, `make test`, and `make spec`.

The package list is already at its exact 1,300-file ceiling, so add no file.
Keep `src/render/json.rs` at its pinned 1,555-line baseline,
`src/cli/variant/dispatch.rs` at or below its 700-line cap, and all unpinned
Rust source files at or below 1,000 lines. In particular,
`src/render/markdown/variant/tests.rs` starts at 995 lines and must not grow
past that rail; place broader contracts in existing external test files or
replace weaker coverage.

## Dependencies

None. Ticket 1094 is already complete and the current variant identity fixture
already serves the negative and positive anchors needed here.

## Review

- Design review: **ACCEPT with no material findings**. Independent review
  reproduced all negative, positive, broad, and mixed cases; confirmed the
  shared CLI/raw/typed MCP path, deterministic map behavior, unreleased
  output-only compatibility boundary, and package/source rails.
- Code review: **ACCEPT after remediation**. The implementation was accepted;
  review required one additional exact JSON byte-parity assertion. Final
  re-review confirmed raw/typed Markdown and JSON byte parity, retained
  structural assertions, accurate record wording, scoped changes, and clean
  rails with no remaining findings.

## Implementation evidence

- Red: the focused model contract failed with the former
  `{gene:"unresolved", hgvsp:"resolved", residue_alias:"resolved"}` values
  instead of the required unavailable/evaluated vocabulary.
- Green: the focused model contract and shared Markdown/JSON renderer contracts
  pass, including mixed states, canonical ordering, the non-null empty JSON map,
  broad-search identity nullability, and absence of the retired output names.
- Repository gates: `make lint` passed; `make test` passed 3,149 Rust tests (30
  skipped), 893 Python contracts (3 skipped), and strict documentation;
  `make spec` passed every routine group, including the deterministic identity
  checks and raw/typed MCP Markdown and JSON parity, plus 39 parallel-isolation
  and 8 static cases.
- Rails: the package remains exactly 1,300 files; `src/render/json.rs` remains
  1,555 lines, `src/cli/variant/dispatch.rs` remains 699 lines, and
  `src/render/markdown/variant/tests.rs` remains 995 lines. `git diff --check`
  passes and no file or dependency was added.

## Code-review remediation

- Finding: the MCP contract compared raw and typed JSON only after parsing,
  leaving byte-for-byte JSON parity unproved; its existing byte-for-byte
  Markdown assertion was already sufficient.
- Red: independent review rejected the missing exact JSON-text assertion.
- Green: the affected MCP contract now asserts `typed_json == raw_json` before
  retaining the structural JSON and vocabulary assertions. Focused
  `make spec-contracts` passed, then `make lint`, `make test` (3,149 Rust tests,
  30 skipped; 893 Python contracts, 3 skipped; strict docs), and `make spec`
  all passed again.

## Completed 2026-09-05

Variant search output now separates exact identity resolution from per-filter
evaluation consistently across CLI, raw MCP, and typed MCP Markdown and JSON.
The primary-agent verification passed `make lint`, `make test` (3,149 Rust
tests with 30 skipped, 893 Python contracts with 3 skipped, and strict docs),
and `make spec` (all routine groups, 39 parallel-isolation cases, and 8 static
cases).
