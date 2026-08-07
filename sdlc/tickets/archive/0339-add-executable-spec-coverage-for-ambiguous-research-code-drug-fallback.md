---
flow: quickfix
priority: 4
---
# Add executable-spec coverage for ambiguous research-code drug fallback

Ticket 310 shipped the canonical `MK-3475 -> pembrolizumab` research-code rescue with a contract assertion. The acceptance criterion that sparse research-code lookups with non-unique discovery signal must fall back to the existing alias-guidance surface (rather than rendering a misleading sparse card) has no executable-spec coverage. Build verified the runtime by inspection but cannot pin the contract.

Completed under March on 2026-04-29, as March ticket 339. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/339-add-executable-spec-coverage-for-ambiguous-research-code-drug-fallback
