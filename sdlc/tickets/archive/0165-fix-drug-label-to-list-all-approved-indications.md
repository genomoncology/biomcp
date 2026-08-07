---
flow: quickfix
priority: 7
---
# Fix drug label to list all approved indications

The `get drug <name> label` summary line only extracts the first approved indication. Thalidomide's label shows "newly diagnosed multiple myeloma (MM)" but the raw FDA label text lists both MM and erythema nodosum leprosum (ENL). When agents use the label subcommand without `--raw`, they see an incomplete picture. The `--raw` output has the full text but is 15K+ chars — too large for efficient agent consumption.

Completed under March on 2026-04-10, as March ticket 165. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/165-fix-drug-label-to-list-all-approved-indications

The landed commit range could not be recovered from git, so no
record accompanies this entry. The work products above are the
evidence that survives; the absence of a record is a gap in what
git can still prove, not a sign the work is missing.
