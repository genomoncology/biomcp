---
flow: build
priority: 3
---
# Harden biomcp-ci against stale binary fallback in repo-local verification

Two independent verify passes found the same operator-trust problem: `tools/biomcp-ci` can exercise a stale installed `biomcp` when `BIOMCP_BIN` is unset. In tickets 355 and 357, manual verification accidentally hit `/home/ian/.cargo/bin/biomcp` rather than the worktree release binary and observed stale behavior.

Completed under March on 2026-04-30, as March ticket 363. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/363-harden-biomcp-ci-against-stale-binary-fallback-in-repo-local-verification
