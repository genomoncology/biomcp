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
2. Saturated gate-host results (12 spinners):
   - `tests/test_routine_fixture_recovery.py` — 61 passed in 40.26 s, rc 0
     (the 9c689fad reaping-wait fix holds).
   - drug.md single-page — 15 passed, rc 0, re-confirmed.
3. **BLOCKER — the per-page scoping regresses gene.md.** Same single-page
   configuration, same host, same saturated settings:
   - old code (6a4b1a1a): gene.md 22 passed, rc 0, in 29 s.
   - new code (9c689fad): gene.md 15 passed, **7 failed** — blocks at lines
     518, 529, 548 (Partial ClinGen evidence), 575, 605, 628 (GenCC
     submission-level validity / adapter projection parity), and 714
     (GenCC health), with `expected true / actual false` and empty parity
     output. Reproduced twice under spinners, once with no spinners, and in
     the full untrimmed lane at the branch tip with no spinners
     (`bash scripts/run-specs.sh spec`, rc 1, same seven blocks).
   - The failures are functional, not marginal timing: they reproduce with
     zero load, and the same blocks pass at old code under 12 spinners.
     The likely seam is the interaction between the worker-namespaced base
     and the ClinGen/GenCC download flows (both are download-path flows, and
     both are the only degraded/synthetic-response families in the page).
4. Full spec mode under 12 spinners: deferred to the parent's merged
   saturated gate; note item 3's full-lane no-spinner run already fails at
   the tip, so that gate would be red as-is.

Decision note: the `timeout=600` fence on the parity block at
`spec/entity/gene.md:628` (added by ticket 1194) is retained. The block runs
three representatives across CLI, raw MCP, typed MCP, and batch; the directive
is appropriate where it stands, and `spec/entity/gencc.md` does not exist. The
line-714 health block needed no budget change — its failure was shared-log
interference, which commit a78584a6 removes.

Recommendation: do not merge the branch as-is. Bisect the regression between
`scripts/run-specs.sh` (per-page base rewrite) and
`spec/fixtures/setup-provider-contract-spec-fixture.sh` (namespace routing) —
the focused namespace test still passes, so the fixture routing alone is
suspect only in combination with the rewritten base for download flows. A
minimal next experiment is to pin `BIOMCP_CLINGEN_BASE` and the GenCC download
base to the unprefixed fixture base in the rewrite exclude list and re-run the
gene page.

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
