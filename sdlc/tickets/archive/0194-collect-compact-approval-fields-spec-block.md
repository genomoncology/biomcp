---
flow: quickfix
priority: 4
---
# Collect Compact Approval Fields spec block

`spec/05-drug.md:236` has a `Compact Approval Fields` section that documents the drug approval JSON contract — `approval_date`, `approval_date_raw`, `approval_date_display`, `approval_summary`. The bash block inside the section uses only `jq -e` assertions with no `mustmatch` directive, so the markdown spec collector silently skips it. Running `uv run --extra dev pytest spec/05-drug.md --collect-only -q | grep -i approval` returns only `Human-Friendly Approval Date (line 253)`; the adjacent `Compact Approval Fields (line 236)` block is not registered as a `BashItem`. The approval JSON contract is documented but unverified — a regression that drops any of those fields would slip through `make spec` and `make spec-pr` today.

Completed under March on 2026-04-16, as March ticket 194. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/194-collect-compact-approval-fields-spec-block
