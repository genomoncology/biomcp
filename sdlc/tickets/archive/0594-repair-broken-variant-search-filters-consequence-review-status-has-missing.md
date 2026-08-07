---
flow: build
priority: 9
---
# Repair broken variant-search filters: --consequence, --review-status, --has/--missing

An adversarial input-fuzz sweep (2026-07-18, `experiments/161-biomcp-adversarial-input-fuzz/FINDINGS.md`) found three variant-search filters broken or unvalidated — the confident-emptiness anti-pattern reachable through **documented usage**. All confirmed live against the release binary:

Completed under March on 2026-07-18, as March ticket 594. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/594-repair-broken-variant-search-filters-consequence-review-status-has-missing
