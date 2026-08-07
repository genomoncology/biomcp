---
base: 8c9d58c2a3bf3531f83b916b468ad251f407a8c7
head: 029be9c372d96edb5749f4a32b7cadc4b04c8aff
---
Two independent verify passes found the same operator-trust problem: `tools/biomcp-ci` can exercise a stale installed `biomcp` when `BIOMCP_BIN` is unset. In tickets 355 and 357, manual verification accidentally hit `/home/ian/.cargo/bin/biomcp` rather than the worktree release binary and observed stale behavior.

Imported from March ticket 363. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/363-harden-biomcp-ci-against-stale-binary-fallback-in-repo-local-verification
