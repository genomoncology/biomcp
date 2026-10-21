---
flow: build
priority: 5
---

# A ClinicalTrials.gov total the provider did not supply is reported as exactly zero

## Outcome

`biomcp search trial --count-only` reports a ClinicalTrials.gov total omitted
from the response as unknown on JSON, text, and raw MCP surfaces. A provider
total that is present keeps its current exact or age-qualified approximate
meaning.

## Verified current facts

The wire model is already truthful. `CtGovSearchResponse.total_count` is
`Option<u32>` at `src/sources/clinicaltrials.rs:336-340`; serde therefore
decodes an omitted `totalCount` as `None`. The loss occurs in the fast count
path at `src/entities/trial/search/ctgov.rs:944-957`:

```rust
let total = resp.total_count.unwrap_or(0) as usize;
return Ok(ctgov_count_from_native_total(total, filters.age.is_some()));
```

`ctgov_count_from_native_total` at `src/entities/trial/search/ctgov.rs:639-645`
can currently produce only `Exact(total)` or `Approximate(total)`. Thus an
omitted value becomes `Exact(0)` without `--age` and `Approximate(0)` with
`--age`.

The age case reaches this same fast path. `prepare_ctgov_search_context` at
`src/entities/trial/search/mod.rs:318-340` marks only facility/geo verification
and eligibility-keyword filters as expensive; age alone leaves
`uses_expensive_post_filters == false`.

`TrialCount` is the crate-internal entity boundary at
`src/entities/trial/mod.rs:257-265`. The enum and `count_all` are declared
`pub` inside the trial module, but `src/lib.rs` keeps the enclosing `entities`
module private, so neither item is part of BioMCP's externally reachable Rust
API. Changing the enum still requires updating every crate-internal exhaustive
match, but it is not a downstream source-compatibility break. `TrialCount`
also has no serde derive: its Rust shape is not the JSON contract. The
count-only renderer builds the public JSON object explicitly.

`count_all` carries the enum directly from the CTGov branch at
`src/entities/trial/search/mod.rs:363-369`. The count-only CLI renderer at
`src/cli/trial/dispatch.rs:208-235` then maps an unknown count to `total: null`
in JSON and to a text line. The raw MCP `biomcp` tool uses the same parsed CLI
and renderer through `src/mcp/shell.rs:340-358` and
`src/cli/outcome.rs:683-695`; MCP does not have a separate count conversion.
The typed MCP `search` schema at `src/mcp/shell.rs:191-209,235-280` does not
offer `count_only`, so it is not a surface for this behavior.

The current `Unknown` model needs one correction beyond the original ticket
wording. It already has more than one cause:

- `completed_ctgov_union_count` returns it for degraded alias coverage at
  `src/entities/trial/search/ctgov.rs:665-670`;
- `count_all_with_ctgov_union` returns it when its traversal cap would be
  exceeded at `:870-872`; and
- the expensive single-query traversal returns it at its cap at `:964-967`.

Nevertheless, `src/cli/trial/dispatch.rs:232-234` renders every one as
`Total: unknown (traversal limit reached)`. That reason is true for the two cap
paths but not for degraded coverage, and it would also be false for a missing
provider total. The implementation must preserve reasons rather than treating
bare `Unknown` as synonymous with a cap.

Focused baseline on 2026-09-04 against `0.9.0-dev.6`:

```text
cargo test --no-default-features entities::trial::search::ctgov::tests::count_all_returns -- --nocapture
3 passed; 0 failed
```

Those existing tests cover present totals (`Approximate(250)` with age and
`Exact(494)` without it) and the cap predicate, but not an omitted first-page
total or rendered unknown reasons.

## Deterministic provider evidence and its limit

`testdata/sources/ctgov/search_phelan_next_20260811.json` has `studies` and
`nextPageToken` but no `totalCount`. Its SHA-256 is
`7cfc14e8ff4bbb97531dea0ab938c740ed7112aff1be047dea81d9592e352b82`, matching
`testdata/sources/capture-receipts.json` `entries[219]` (the 220th entry). That
receipt classifies it `real_and_receipted`, records a request containing both
`countTotal=true` and a `pageToken`, and says minimization preserved the
omitted later-page total.

This proves that CTGov can omit a requested total. It is a later-page capture,
whereas the defective fast count path requests the first page with page size
one. No committed first-page capture demonstrates that omission. Do not serve
the later-page bytes for a synthetic first-page request or describe them as a
first-page capture. A local fixture response such as `{"studies":[]}` is
acceptable when clearly identified as a synthetic reproduction of the
observed optionality; it does not need a capture receipt.

## Accepted design and exact scope

Model the cause at the `TrialCount` entity boundary rather than inferring it in
the renderer. In `src/entities/trial/mod.rs`, give the unknown state an explicit
reason for each of the three current producer meanings:

- provider omitted the requested total;
- traversal limit reached; and
- expanded CTGov coverage was incomplete/degraded.

The exact Rust type and variant names are implementation details. Because this
enum is not externally exported or serialized, a reason-bearing unknown state
does not alter a public Rust or serde enum representation. A bare unknown value
that makes the renderer guess is not acceptable. Update every producer in
`src/entities/trial/search/ctgov.rs`:

- map `resp.total_count == None` in `count_all_with_ctgov_client` to the
  provider-omission reason before applying age precision;
- retain `Exact(n)` / `Approximate(n)` for `Some(n)`;
- attach the traversal reason at the two cap returns; and
- attach the incomplete-coverage reason in `completed_ctgov_union_count`.

Update the count-only branches in `src/cli/trial/dispatch.rs` so all unknown
reasons serialize as `{"total": null}` with no `approximate` member. Text must
remain `Total: unknown (traversal limit reached)` for cap outcomes and use a
truthful, cause-specific nonnumeric explanation for provider omission and
incomplete coverage. Do not add a guessed number.

Do not change `CtGovSearchResponse`, request construction, `countTotal=true`,
page size, paging, filtering, alias fan-out, ordinary trial search results, or
the NCI branch.

## Test targets and red-before-green proof

The first regression must exercise the current API and fail by assertion, not
fail to compile after changing a helper signature. Add an async local HTTP
fixture in `src/entities/trial/search/ctgov/tests.rs` (or an equally narrow
existing test seam), point `BIOMCP_CTGOV_BASE` at it under
`#[serial_test::serial(source_env)]`, return the clearly synthetic body
`{"studies":[]}`, and call `count_all_with_ctgov_client`. Before the fix, the
assertions that the result is unknown fail as `Exact(0)` and `Approximate(0)`.
Cover both default filters and `age: Some(0.5)`. Also assert that the request
still contains `countTotal=true` and `pageSize=1`.

Then add focused tests for these contracts:

- in `src/entities/trial/search/ctgov/tests.rs`, a present total remains
  `Exact(n)` without age and `Approximate(n)` with age; missing is unknown in
  both cases; cap and degraded-coverage producers carry their distinct reasons;
- in `src/cli/trial/tests.rs`, the count-only JSON renderer emits
  `total: null` and no `approximate` field for every unknown reason, while
  present approximate totals retain `approximate: true`;
- in `src/cli/trial/tests.rs`, text rendering preserves the literal cap line
  and does not claim either zero or a traversal cap for provider omission;
  incomplete coverage likewise must not claim a traversal cap; and
- exercise the raw `biomcp` tool through the existing in-module
  `BioMcpServer::biomcp(Parameters(ShellCommand { .. }))` seam in
  `src/mcp/shell.rs`, as the current raw-MCP rejection tests do (JSON or text is
  sufficient). Calling `crate::cli::execute_mcp` instead is not a raw-tool
  test: it bypasses shell splitting, allowlisting, tool-result construction,
  and MCP sanitization. No separate typed-MCP test is required because typed
  search cannot request count-only.

If renderer logic is extracted from `handle_search` to make these assertions
pure, keep it in `src/cli/trial/dispatch.rs`; the owning layer remains the trial
count-only renderer.

## Acceptance

- A CTGov count response without `totalCount` yields JSON `total: null`, with
  no `approximate: true`, both with and without `--age`.
- Text for that response states an unknown total and accurately says the
  provider omitted it; it contains neither `Total: 0` nor `traversal limit
  reached`.
- A present total retains its number. With age it retains the existing
  approximate qualification; without age it remains exact.
- A traversal-capped count still renders exactly `Total: unknown (traversal
  limit reached)`.
- Degraded alias coverage is unknown without being mislabeled as a traversal
  cap.
- CLI and raw MCP use the same outcomes. Typed MCP remains unchanged.
- Focused trial entity and CLI tests pass, followed by `make lint`, `make test`,
  and `make spec`.

## Dependencies and overlap

Dependencies: none. The optional source field, `TrialCount` boundary, local
base-URL override, and CLI/MCP execution seams already exist.

The synthetic first-page response belongs inline in a local transport test,
not under `testdata/sources`. Ticket 1126's fixture-key audit covers selected
trial records passed to the conversion layer; transport-envelope keys such as
`studies`, `nextPageToken`, and `totalCount` are explicitly outside that
provider-record selector. The inline `{"studies":[]}` response therefore does
not require a capture receipt or fixture-key inventory entry, but it must stay
labeled synthetic and must not replace or masquerade as the receipted
later-page response.

Ticket 1103 concerns recovery commands for degraded/capped sections; it does
not own count precision or unknown-reason rendering and is not a blocker. Ticket
1023 concerns zero counts for unavailable enrichment sources and does not share
this trial count path.

The NCI fallback at `src/entities/trial/search/mod.rs:370-373` is explicitly out
of scope. The two committed NCI search payloads contain top-level totals (2094
and 2112); one is pending provenance verification and the 2026-08-11 capture is
`real_and_receipted`. There is no deterministic NCI omitted-total reproducer in
this repository, so this CTGov evidence does not justify changing NCI behavior.

## Implementation evidence

Implemented 2026-09-04. `TrialCount::Unknown` now carries a provider-omission,
traversal-limit, or incomplete-coverage reason. The CTGov fast count path maps
an absent `totalCount` directly to provider omission before considering age
precision; present totals still become exact without age and approximate with
age. Both traversal-cap returns and the degraded alias-union return now retain
their own reasons. The count-only renderer emits `total: null` without an
`approximate` member for every unknown and uses cause-specific text, while the
NCI branch, typed MCP schema, paging, filtering, and request construction are
unchanged.

Red proof was established against the current API before changing production
code:

```text
cargo test --no-default-features count_all_keeps_an_omitted_provider_total_unknown -- --nocapture
omitted total must remain unknown, got Exact(0)
test result: FAILED. 0 passed; 1 failed
```

The regression uses an inline synthetic `{"studies":[]}` first-page transport
envelope, labels it synthetic in code, and asserts both `countTotal=true` and
`pageSize=1`. It is not stored under `testdata/sources` or represented as a
provider capture. Focused green validation:

```text
cargo test --no-default-features --lib entities::trial::search::ctgov::tests -- --nocapture
31 passed; 0 failed

cargo test --no-default-features --lib traversal_limit_reason_at_its_cap -- --nocapture
2 passed; 0 failed

cargo test --no-default-features --lib cli::trial::tests -- --nocapture
39 passed; 0 failed

cargo test --no-default-features raw_biomcp_tool_preserves_an_omitted_ctgov_total_as_null -- --nocapture
1 passed; 0 failed

cargo clippy --locked --no-default-features --lib --tests -- -D warnings
passed

uv run --no-sync python tools/check-source-capture-receipts.py --root testdata/sources
passed

tools/check-quality-ratchet.sh
passed

make spec-static
8 passed

cargo package --list --allow-dirty --no-verify | wc -l
1300

TMPDIR="$PWD/.cache/package-boundary-tmp" uv run --no-sync pytest \
  tests/test_source_package_boundary.py -q \
  --basetemp "$PWD/.cache/package-boundary-tmp/pytest"
6 passed
```

`cargo fmt --all -- --check` and `git diff --check` also pass. The primary agent
owns the final repository-wide `make lint`, `make test`, and `make spec` gates.
Implementation files are `src/entities/trial/mod.rs`,
`src/entities/trial/search/ctgov.rs` and its focused tests,
`src/cli/trial/dispatch.rs` and its focused tests, and `src/mcp/shell.rs` for the
actual raw-tool seam. Coverage is kept in existing focused sidecars plus one
ticket-scoped raw-MCP module so the CLI and Rust source-size ratchets remain
satisfied without exhausting the package file budget.

The primary `make test` run then passed all 3,090 Rust tests but exposed a
package-boundary failure: the three initially added ticket test files raised
the Cargo package inventory from 1,299 to 1,302 files, above the fixed 1,300
limit. The behavioral coverage was consolidated without weakening that limit:
entity count tests now live in the existing CTGov test sidecar, their reusable
fixture support lives in the existing trial test-support sidecar, CLI renderer
assertions live inline in their owning dispatch module, and the sole new file
contains only raw-MCP coverage. The final package inventory contains exactly
1,300 files; no package exclusions, limits, fixture ratchets, or unrelated
content were changed.

## Review

- Independent design review (2026-09-04): REVISE, then ACCEPT after amendment.
  The bug, receipt index/hash, first-page evidence limit, and three existing
  producer causes were verified. The amendment corrects the mistaken implication
  that `TrialCount` is an externally public/serialized API, records why explicit
  reasons are compatible and necessary, ties the synthetic envelope to ticket
  1126's actual selector boundary, and replaces the false raw-MCP test claim
  with the existing `BioMcpServer::biomcp` tool seam. Degraded alias coverage
  remains in scope: it is already an unknown producer, and making the boundary
  reason-bearing requires classifying it rather than preserving a known false
  traversal explanation.
- Independent code review (2026-09-04): REVISE. The first implementation
  asserted cap predicates and the shared reason constant independently, so it
  did not prove that either actual cap producer returned the traversal reason.
  Remediated by injecting the unchanged production cap into the private CTGov
  count implementations: the public path still supplies 50, while focused
  tests supply a tiny cap and execute the actual expensive-single-query and
  alias-union cap branches. Both assert
  `Unknown(TraversalLimitReached)` and zero provider requests at the boundary.
  The superseded predicate/constant-only assertions were removed.
- Independent remediation review (2026-09-04): ACCEPT with no findings. Both
  real cap returns are exercised; the sole production entry point supplies the
  unchanged cap of 50 and preserves the original `>= 50` single-query and
  `fetched + active > 50` union semantics. Test seams remain crate-private,
  focused entity/CLI/raw-MCP tests, formatting, Clippy, the quality ratchet,
  and `git diff --check` all passed.
- Second independent code review (2026-09-04): REVISE. Consolidation had
  removed the local CTGov fixture and explicit zero-request assertions from
  both cap tests, and placed pure CLI renderer tests behind a cross-layer
  test-only re-export. Remediated by installing the shared local CTGov fixture
  under `serial(source_env)` in both actual-producer tests and asserting each
  request log remains empty. Renderer tests now live in an inline
  `#[cfg(test)]` module in `src/cli/trial/dispatch.rs`; the CLI re-export and
  MCP dependency were removed.
- Final independent remediation review (2026-09-04): ACCEPT with no findings.
  Verified both real cap returns, empty loopback request logs, serialized RAII
  environment restoration, the unchanged production cap of 50, private
  CLI-owned renderer tests, and raw-MCP-only scope for the sole new file.
  Focused entity/renderer/raw-MCP tests, formatting, the quality ratchet, and
  the locked/offline 1,300-file package inventory all passed.

## Completed 2026-09-04

ClinicalTrials.gov count responses that omit `totalCount` now remain unknown
before age precision is applied. Unknown counts carry truthful provider,
traversal-cap, or incomplete-coverage reasons; JSON remains `total: null`, and
CLI/raw-MCP text no longer invents zero or mislabels the cause. Present exact
and age-qualified approximate totals are unchanged.

Final primary-agent verification passed: `make lint`; `make test` (3,090 Rust
tests passed with 30 skipped, 883 Python tests passed with 3 skipped, the
1,300-file Cargo package boundary passed, and strict documentation built); and
`make spec` (all routine pages, 38 parallel-isolation contracts, and 8 static
specs passed).
