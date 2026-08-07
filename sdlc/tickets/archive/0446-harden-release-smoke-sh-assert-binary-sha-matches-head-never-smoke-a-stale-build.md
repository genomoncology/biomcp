---
flow: quickfix
priority: 4
---
# Harden release-smoke.sh: assert binary SHA matches HEAD, never smoke a stale build

release-smoke.sh defaults to an existing target/release binary and only\ \ rebuilds if absent, so it can validate a stale build (it did \u2014 a dab68f67-stamped\ \ binary gave a misleading 444 FAIL). Assert binary SHA == HEAD; rebuild if missing/stale.\ \ Low-priority tooling hygiene.

Completed under March on 2026-06-24, as March ticket 446. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/446-harden-release-smoke-sh-assert-binary-sha-matches-head-never-smoke-a-stale-build
