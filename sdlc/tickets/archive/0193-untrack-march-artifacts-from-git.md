---
flow: quickfix
priority: 6
---
# Untrack .march artifacts from git

Two `.march/` runtime artifacts are tracked in git by accident: `.march/code-review-log.md` and `.march/publish-report.md`. Both are per-run step outputs that leak review artifacts from one branch into another branch's diff. They were committed before `.march/` was added to `.gitignore`.

Completed under March on 2026-04-15, as March ticket 193. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/193-untrack-march-artifacts-from-git
