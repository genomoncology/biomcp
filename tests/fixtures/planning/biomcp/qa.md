# BioMCP QA

## Review-driven checks for source expansions

When a ticket adds or expands a source-backed section, cover all of the
following before the ticket is considered done:

- `--help`, `list <entity>`, and docs all describe the same section
  availability and query requirements.
- Base `get` cards stay concise; requested sections show truthful
  unsupported, empty, or unavailable states in both human and `--json`
  output.
- Suggested next commands and `_meta.next_commands` are source-aware, do not
  repeat the current command, and do not suggest unsupported sections.
- At least one spec or targeted regression test covers the main user flow and
  at least one empty or unsupported-state case.
- Evaluate `biomcp health` and `scripts/contract-smoke.sh` deliberately: add
  the source when operators need readiness signals or live contract probes,
  otherwise document why it stays out.

## Current attention areas

- Pathway source-aware sections and default-card weight after KEGG expansion.
- Protein complexes terminal layout and next-command usefulness.
- Operator readiness coverage for KEGG, HPA, ComplexPortal, and g:Profiler.
- CLI validation consistency for missing queries and invalid sections.
