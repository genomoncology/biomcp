---
flow: build
priority: 5
---
# Trim remaining operator-facing CLI/help noise before v0.8.18

The pre-release review surfaced several smaller discovery problems that are not release-blocking individually but add noise to the shipped CLI: generic `serve-http` flags in help, a visible removed `serve-sse` compatibility command, and dense BioASQ ingest help/lane naming.

Completed under March on 2026-03-26, as March ticket 054. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/054-operator-help-cleanup

The landed commit range could not be recovered from git, so no
record accompanies this entry. The work products above are the
evidence that survives; the absence of a record is a gap in what
git can still prove, not a sign the work is missing.
