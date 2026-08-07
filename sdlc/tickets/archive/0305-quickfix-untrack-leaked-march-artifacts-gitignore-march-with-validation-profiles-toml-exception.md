---
flow: quickfix
priority: 5
---
# Quickfix: untrack leaked .march/ artifacts; gitignore .march/* with validation-profiles.toml exception

Standing orders say `.march/` must be gitignored. The biomcp repo currently has `.march/` in `.gitignore`, but `.march/code-review-log.md` and `.march/validation-profiles.toml` were committed before the ignore was in place and remain tracked.

Completed under March on 2026-04-25, as March ticket 305. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/305-quickfix-untrack-leaked-march-artifacts-gitignore-march-with-validation-profiles-toml-exception

The landed commit range could not be recovered from git, so no
record accompanies this entry. The work products above are the
evidence that survives; the absence of a record is a gap in what
git can still prove, not a sign the work is missing.
