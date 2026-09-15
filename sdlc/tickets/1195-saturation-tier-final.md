---
flow: build
priority: 1
deps: []
---

# 1195: The provider request log is scoped per consuming spec page

## Goal

Every spec page that observes `BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG` gets its
own request log, so page assertions are deterministic regardless of what other
pages run concurrently, and the saturated full-spec lane stops failing on
count windows that other pages perturb.

## Current Facts

The overnight saturated run at main 6a4b1a1a failed `spec/entity/drug.md`
("Card command discovery", block at line 35) and `spec/entity/gene.md`
("GenCC adapter projection parity", block at line 714) while the Rust suite
passed 3566/3566. Instrumented reproduction (full spec mode, 12 CPU spinners,
main 6a4b1a1a) captured the mechanism:

- One provider-contract fixture serves every page; four pages read its single
  mutable request log: disease, drug, gene, trial.
- drug.md's block truncates that shared log (`reset_log`) between phases and
  then compares absolute counts (`wait_for_count`). With other pages writing
  mid-window, the captured counts were 8 (expected 5), 7 (expected 4), and
  20 vs 17 between the JSON and Markdown phases — the batch comparison failed:
  `DIAG batch_md_count 17 batch_json_count 20`.
- gene.md's blocks delta-count the same shared log ("so concurrent specs
  cannot perturb", per its own comment) and still failed: drug.md's truncation
  inside a delta window makes the delta wrong, and the line-714 health block's
  absolute count of the GenCC download GETs is truncated the same way. In the
  same reproduction gene.md failed at its line-23 block with empty output —
  its final delta test (`-eq 0`) after a mid-window truncation.

The ctgov fixture already solves exactly this for its pages with a per-page
worker namespace (`prepare_ctgov_page_request_log`,
`__biomcp_ctgov_worker/<request-log.XXXXXX>`); the provider fixture had no
equivalent.

One premise correction: the failing "GenCC adapter projection parity" report
at gene.md line 714 is a plain health/count block, not the parity block; the
parity block lives at gene.md:628 and already carries `timeout=600` from
ticket 1194 (it is genuinely long-running, and the edit is appropriate where
it stands — `spec/entity/gencc.md` does not exist). The line-714 failure was
interference, not a budget miss, and needs no timeout change.

## Change

- `spec/fixtures/setup-provider-contract-spec-fixture.sh`: the server accepts
  `/__biomcp_provider_worker/<request-log.XXXXXX>/...`, strips the prefix for
  routing, logs to that file in the fixture root (404 for an unknown
  namespace), and exports `BIOMCP_PROVIDER_CONTRACT_ROOT`. Unprefixed requests
  keep logging to the shared log exactly as before.
- `scripts/run-specs.sh`: `provider_page_consumes_request_log` and
  `prepare_provider_page_request_log` mirror the ctgov pair: per-page
  `mktemp` log, and every exported `BIOMCP_*` endpoint variable whose value
  starts at the fixture base rewritten to the worker base. Two variables are
  deliberately not scoped: `BIOMCP_PROVIDER_CONTRACT_BASE` (and
  `BIOMCP_TEST_UNPACED_ORIGIN`) stay at the unprefixed fixture origin because
  the spec pages derive the unpaced signal from the base inline, and both
  consumers of that signal require a bare origin (scheme+host+port, path
  "/"): the rate limiter's unpaced bypass
  (`UnpacedOrigin::parse_signal`) and the GenCC fixture-override gate
  (`fixture_override_allowed`, required because the spec profile inherits
  release and `debug_assertions` is off). A worker-prefixed signal silently
  disables both, which re-enables 100 ms pacing and denies the GenCC
  override; that was the exact gene-page regression. Pages that compose
  sub-bases from the plain base log to the shared request log, which no page
  reads any more, so every asserted entry still lands in the consuming
  page's private log. Called in the per-page subshell of the parallel lane
  beside the ctgov call.
- `tests/test_provider_contract_fixture.py`: focused test proving the
  namespace route logs to its private file, leaves the shared log untouched,
  keeps unprefixed logging on the shared log, and 404s an unknown namespace.
- `tests/test_routine_fixture_recovery.py:740`: the reaping test's runner
  wait widens from 3 s to 60 s (an observation budget like its siblings, not
  a latency assertion); under 12 spinners the 3 s budget expires.

No production code, no page content, no production constants change.

## Verification

1. Focused on the dev host: `uv run --no-sync pytest
   tests/test_provider_contract_fixture.py
   tests/test_routine_fixture_recovery.py -q` — 64 passed in 12.21 s
   (3 provider-fixture tests including the new namespace test, plus the 61
   runner-lifecycle tests).
2. Saturated gate-host results (12 spinners):
   - `tests/test_routine_fixture_recovery.py` — 61 passed in 40.26 s, rc 0
     (the 9c689fad reaping-wait fix holds).
   - drug.md single-page — 15 passed, rc 0, re-confirmed.
3. Resolution of the gene-page regression (this is the refined mechanism):
   the original rewrite replaced `BIOMCP_PROVIDER_CONTRACT_BASE` itself with
   the worker-prefixed URL, and the spec pages derive
   `BIOMCP_TEST_UNPACED_ORIGIN` from that base inline. Both consumers of the
   signal require a bare origin (path "/"): the rate limiter's unpaced
   bypass silently stopped matching, re-enabling 100 ms pacing (the ClinGen
   blocks' 40–200 ms optional budgets starved before their fixture requests
   left), and the GenCC fixture-override gate silently denied the
   `BIOMCP_GENCC_BASE` override (the release-derived spec profile turns the
   `debug_assertions` short-circuit off), so GenCC falls back to the real
   endpoint and returns empty offline. The fix keeps the base and the signal
   unprefixed and scopes only the endpoint variables.
4. Proof results at the refined tip (dev host, single-page runner):
   - gene.md — 22 passed, rc 0, no spinners; 22 passed, rc 0, under 12
     spinners.
   - drug.md — 15 passed, rc 0, under 12 spinners.
   - `tests/test_routine_fixture_recovery.py` — 61 passed, rc 0; together
     with the three provider-fixture tests, 64 passed, rc 0, under 12
     spinners.
5. Full parallel routine lane after the refinement on the dev host (all
   pages, four workers, no spinners): every page green, including gene.md
   22/22 and drug.md 15/15; zero FAIL lines, python contracts 39 passed. The
   parent's merged saturated gate remains the acceptance run.
6. Post-remediation verification (dev host, b0b102a4):
   - `uv run --no-sync pytest "tests/surface/test_parallel_isolation_contract.py" -q`
     — 41 passed, including the previously-failing
     `test_ctgov_parallel_pages_receive_private_mutable_logs` (the P0
     confirmation), the new provider isolation test, and the new rewrite-set
     test.
   - `uv run --no-sync pytest tests/test_provider_contract_fixture.py -q` —
     3 passed; `tests/test_routine_fixture_recovery.py -q` — 61 passed.
   - gene.md single page — 22 passed, rc 0; drug.md single page — 15 passed,
     rc 0.
   - `tools/check-quality-ratchet.sh` — exit 0; `tools/check-shell-workflows`
     — 89 Bash files syntax-checked, 80 with ShellCheck at warning severity,
     4 workflows; `ruff check` clean on the changed test file.

## Complexity

- Contract score: 1 (several explicit cases: namespace routing, per-page
  scoping, shared-log fallback, unknown-namespace 404, env rewrite set)
- State and timing score: 1 (test-harness observation partitioning; the
  server stays stateless per request; no concurrent protocol changes)
- Reach score: 1 (runner, fixture, and their focused tests)
- Proof score: 1 (focused tests plus saturated single-page and full-lane
  runs on the gate host)
- Cost of error score: 1 (test infrastructure only; a wrong rewrite set
  would break spec pages visibly and locally)
- Total: 5
- Minimum level floor: none (the change removes a test-harness interference
  rather than introducing concurrent state management)
- Final level: 2
- Reasons: deterministic test-harness partitioning with saturated proofs
- Selected model: implemented in the dispatched level-2 workstream

## Review

- Independent code review, first pass at e31d4b16: REJECT.
  - P0: `prepare_provider_page_request_log` aborted on an unset
    `BIOMCP_PROVIDER_CONTRACT_ROOT` (the `:?` expansion), breaking
    `tests/surface/test_parallel_isolation_contract.py::
    test_ctgov_parallel_pages_receive_private_mutable_logs` in the make-test
    lane: that harness stubs the provider setup with no env file while
    trial.md consumes both request logs. The spec lane masked it because its
    fixture always configures the environment first.
  - P2-1: no focused test pinned the rewrite set (which variables are
    worker-scoped and which are deliberately kept).
  - P2-2: a missing export indent at
    spec/fixtures/setup-provider-contract-spec-fixture.sh:633.
- Remediation at b0b102a4: the preparation is a clean no-op when the fixture
  root is unset (a configured root without a base still fails loudly,
  mirroring `run_provider_contract_fixture`'s tolerance); the harness gains
  the provider analogue of the CTGov isolation contract — a stub provider
  setup writing `spec-provider-contract-env` plus the private-log assertion
  over the four provider pages — and a focused rewrite-set test pinning that
  endpoint variables are worker-scoped while
  `BIOMCP_PROVIDER_CONTRACT_BASE`, `BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG`,
  `BIOMCP_PROVIDER_CONTRACT_READY_FILE`, and `BIOMCP_TEST_UNPACED_ORIGIN`
  are not rewritten; the indent is fixed. Verification item 6 records the
  run.
- Re-review: pending (parent orchestrator).
