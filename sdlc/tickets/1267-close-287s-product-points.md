# Close GitHub #287's product points

From `sdlc/issues/2026-09-28-review-follow-ups-after-1263-and-1264.md`
(GitHub #287 section). The reporter's four points beyond the build
profile: exit codes, the debug-embed feature, the 0.8.x wheels, and
the open issue itself. This ticket covers the first two; the 0.8.x
wheel check needs PyPI access (the supervisor runs it); the reply
waits for Ian's go and for 0.9.1 to reach PyPI.

## Fix

1. **Exit codes.** On 0.9.0 every asset-missing path printed an
   error and exited 0. A missing skill name already maps to
   `BioMcpError::NotFound` (exit 1) on main — tests now pin that.
   The one remaining mask: `skill list` returned
   `Ok("No skills found")`, exit 0, when the embedded catalog is
   empty. A healthy build always lists skills, so an empty catalog
   means broken asset embedding — it must fail loudly. The empty
   list now returns `NotFound` naming the embedded skills/ tree
   (exit 1). Chart has no listing path to mask: every command
   resolves one embedded doc and a miss already exits 1; tests pin
   the mapping and the healthy paths.
2. **debug-embed.** Decision: enable `rust_embed`'s `debug-embed`
   feature. Reason: the failure mode shipped a broken release
   (debug-profile wheels embedded nothing, and every asset-backed
   feature failed for PyPI consumers); with the feature on, debug
   and release builds embed identically, so `cargo test` — which
   builds debug — exercises the same embedded assets the release
   ships, and no future packaging path can ship a debug binary with
   a runtime lookup root. The dev cost is a rebuild when an
   embedded asset changes, which is rare (skills and chart docs are
   stable). A debug-profile test reads a skill and a chart through
   the embedded path, which only passes when debug embeds.

## Deferred gaps

- The 0.8.x wheel check (PyPI sizes) belongs to the supervisor; its
  finding lands in the #287 disposition.
- The #287 reply itself posts after 0.9.1 reaches PyPI, per Ian's
  earlier approval.

## Review

- Design review: n/a (the review file named the points; the
  debug-embed decision is recorded here with its reason)
- Code review: ACCEPT 2026-09-29 by a fresh worker-context reviewer, dispatch record 9dc2572b-450b-4acc-978f-0eeb8cbc40e0. Verdict verbatim: all three claims verified — the miss and empty-catalog paths return NotFound through catalog.rs:150-158 and :88-96, exit-code wiring proven through error.rs:734-738 and main.rs:206-213; Cargo.toml:93-96 pins debug-embed with the provenance test at tests/test_release_workflow_provenance.py:1360-1372 plus the behavioral debug-embed test; the chart-miss test calls the real embedded_text path chart.rs:239-246. One non-blocking note: the chart mapping pin lives in the skill test module while the real-path proof lives in chart.rs (already labeled). History: the 2026-09-28 ACCEPT on this line was fabricated and reverted; this verdict is the real one.
- Verification: yellow gate green; merged in the 535747d9 chain (full gate on this branch before merge)
