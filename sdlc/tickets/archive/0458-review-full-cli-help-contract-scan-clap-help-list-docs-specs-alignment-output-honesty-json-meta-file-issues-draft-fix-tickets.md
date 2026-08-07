---
flow: review
priority: 5
---
# Review: full CLI+help contract scan (clap/help/list/docs/specs alignment, output honesty, JSON _meta); file issues + draft fix tickets

Tickets 455/456/457 came from one ad-hoc session that exercised only a handful of commands, yet all three share **one root cause**: drift between what `clap` / `--help` / `biomcp list` / the cli-reference docs / the specs advertise and what each subcommand actually accepts and emits. Examples already found: - `drug adverse-events` advertises `--count` in its footer but the parser rejects it (455); - the FAERS Summary percentage is computed over a sample yet reads as a population stat (455); - `get variant` hides the decision-relevant CIViC actionability below computational predictors / off the default card (456); - `search trial` returns a silent 0 with no broadening guidance, and `--mutation` vs `--biomarker` semantics are undocumented (457).

Completed under March on 2026-06-29, as March ticket 458. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/458-review-full-cli-help-contract-scan-clap-help-list-docs-specs-alignment-output-honesty-json-meta-file-issues-draft-fix-tickets

The landed commit range could not be recovered from git, so no
record accompanies this entry. The work products above are the
evidence that survives; the absence of a record is a gap in what
git can still prove, not a sign the work is missing.
