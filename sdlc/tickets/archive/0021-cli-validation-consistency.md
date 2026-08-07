---
flow: build
priority: 6
---
# Normalize CLI usage errors and remediation

The post-expansion review found usage validation drift at the shell boundary: `biomcp search pathway --help` presents `[QUERY]` as optional even though the runtime requires it, the missing-query remediation example is malformed for multi-word queries, and invalid-usage exits are not categorized consistently between clap errors and runtime section errors. This makes scripting and user recovery harder than it should be.

Completed under March on 2026-03-19, as March ticket 021. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/021-cli-validation-consistency

The landed commit range could not be recovered from git, so no
record accompanies this entry. That is a known gap for the
earliest tickets, not a sign the work is missing.
