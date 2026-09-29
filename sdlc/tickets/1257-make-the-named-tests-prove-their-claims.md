# Make the named tests prove their claims

From the 2026-09-27 review file (Tests section). Six tests are
flaky, vacuous, or weaker than their names claim. None needs new
product behavior; each needs an honest test.

## Problem

1. **`the_mcp_path_builds_no_one_shot_runtime`**
   (`src/cli/outcome/probe_tests.rs`) reads the process-wide
   `ONE_SHOT_RUNTIMES_BUILT` counter, which parallel `run_outcome`
   tests also bump, so it can fail at random. It also constructs
   `WorkerDrive::Shared` by hand and never goes through
   `execute_mcp_cli`, so it proves nothing about the MCP path. (The
   1243 review's P2 said the same; the shipped fix marked the sleep,
   not the seam.)
2. **`comments_cdata_and_self_closing_tags_do_not_move_the_depth`**
   (`src/xml.rs`) contains no CDATA and hides three opens against a
   cap of 64 — it cannot fail for the reason its name gives.
3. **The release-mode panic test runs only in `make verify`**
   (`Makefile:113`), the opt-in live lane. It needs no network;
   Ian's order moves it into CI.
4. **`an_allowed_base_keeps_the_path_and_query`**
   (`src/cli/health/http.rs:567`) passes only in debug builds, and
   it and the test at `:553` set a process variable with no serial
   guard.
5. **`tests/tls_ca_bundle_contract.rs:604`** drives two
   `biomcp version` calls. Confirm whether they build an HTTP
   client; if not, the test proves only the startup warning.
6. **GenCC has no wrong-owner or not-a-directory cleanup test**
   (filed 2026-09-26, dropped). The cleanup errno classification
   from 1254 batch 1 has tests only for the deliberate-mismatch and
   retain cases.

## Fix

1. Give the shared-runtime test its own dedicated counter (a
   test-only seam separate from the production counter, or an
   isolated process), and drive it through `execute_mcp_cli` with a
   safe command so the production wiring — `for_shared_call`, the
   MCP entry in `mcp/shell.rs` — is what the test exercises. The
   assertion stays: zero one-shot runtimes built on the MCP path.
2. Rewrite the CDATA test with real CDATA containing text that
   looks like markup (`<![CDATA[<a><b><c>]]>`), and more than 64
   opens hidden inside comments plus CDATA so the test fails if the
   scanner counts them.
3. Move the release-mode panic test into the CI workflow as its own
   step or job on the canonical lane, keeping `make verify` working.
   It must not require network or credentials. Pin it in the
   workflow-contract test the way the other pinned steps are.
4. Make `an_allowed_base_keeps_the_path_and_query` pass in release
   builds honestly (the override must stay release-safe by the
   1254-b2 gating rule: debug_assertions or exact loopback plus the
   test signal), and add a serial guard (lock or single-process
   pattern the repo already uses) for the two process-variable
   tests. If the test cannot be made release-honest without
   weakening the gate, split it: a release-runnable part in CI and
   a debug-only part explicitly named so.
5. Read the TLS test; if `biomcp version` builds no client, extend
   it to a command that does (still offline, fixture CA), and record
   what the two calls actually prove. Do not leave the claim broader
   than the evidence.
6. Add the two GenCC cleanup tests: wrong owner (another uid's
   directory — simulate with an unwritable path the test controls)
   and not-a-directory (a regular file where the directory should
   be), asserting the retain/classify behavior 1254 b1 recorded.

## Review

- Design review: n/a (the review file named each fix)
- Code review: BLOCK once (the CDATA first-text-child assertion
  could never pass; per-family sensitivity; window wording), fixed
  and verified 2026-09-27
- Verification: yellow gate at a9dee294 — lint, test, spec, stress
  OK, zero failed lines (one known GenCC load flake on the first
  run, recorded in its watch issue)

## Acceptance

- The shared-runtime test drives `execute_mcp_cli`, uses a counter
  no other test moves, and is deterministic under the full suite
  (run it repeatedly, e.g. the stress lane shape, in the gate).
- The CDATA test contains CDATA and enough hidden opens to fail if
  the scanner drifts.
- CI runs the release-mode panic test; the workflow contract pins
  it.
- The health tests pass in debug and release, serialized.
- The TLS test's claim matches its evidence, stated in the record.
- GenCC cleanup has both tests.
- Yellow gate green.
