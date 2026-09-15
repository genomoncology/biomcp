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
  `mktemp` log, worker-prefixed base, and every exported `BIOMCP_*` variable
  whose value starts at the fixture base rewritten to the worker base (base,
  log, root, and ready-file variables excluded). Called in the per-page
  subshell of the parallel lane beside the ctgov call.
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
2. Single-page runner under 12 spinners on the gate host: drug.md and gene.md
   (results appended below).
3. Full spec mode under 12 spinners: deferred to the parent's merged
   saturated gate (it runs the whole lane at the merged tip).
4. The reaping test under 12 spinners (result appended below).

Decision note: the `timeout=600` fence on the parity block at
`spec/entity/gene.md:628` (added by ticket 1194) is retained. The block runs
three representatives across CLI, raw MCP, typed MCP, and batch; the directive
is appropriate where it stands, and `spec/entity/gencc.md` does not exist. The
line-714 health block needed no budget change — its failure was shared-log
interference, which commit a78584a6 removes.

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

- Code review: pending (independent review by the parent orchestrator)
