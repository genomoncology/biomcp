---
flow: build
priority: 6
---
# Create standalone BioASQ benchmark harness module

BioMCP now has a proven BioASQ benchmark harness in a research worktree, but it is not yet packaged as a product-owned module. That makes repeat evaluation harder than it should be: model/provider runs, prompt comparisons, session capture, answer submission, and scoring all depend on research-local scripts and operator memory. BioMCP needs a standalone benchmark harness under the repo so the team can run a frozen BioASQ panel end to end, preserve sessions for root cause analysis, and reuse the same substrate for future paper work, longitudinal product tracking, and prompt evaluation without carrying the GEPA machinery along with it.

Completed under March on 2026-03-25, as March ticket 045. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/045-bioasq-benchmark-harness-module

The landed commit range could not be recovered from git, so no
record accompanies this entry. That is a known gap for the
earliest tickets, not a sign the work is missing.
