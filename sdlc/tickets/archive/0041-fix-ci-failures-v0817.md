---
flow: build
priority: 1
---
# Fix CI failures blocking v0.8.17 release PR

PR #227 (`release/v0.8.17`) has 2 failing CI jobs (`contracts`, `spec-stable`) with 4 distinct root causes. All are fixable without feature changes.

Completed under March on 2026-03-26, as March ticket 041. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/041-fix-ci-failures-v0817

The landed commit range could not be recovered from git, so no
record accompanies this entry. That is a known gap for the
earliest tickets, not a sign the work is missing.
