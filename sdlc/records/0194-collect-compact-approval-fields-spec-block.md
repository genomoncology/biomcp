---
base: 52786d96f08a11be9f5bfc686d0f01ea0fc30697
head: d60c17c0a498fce44e97edfc9b9834ee0bd7d750
---
`spec/05-drug.md:236` has a `Compact Approval Fields` section that documents the drug approval JSON contract — `approval_date`, `approval_date_raw`, `approval_date_display`, `approval_summary`. The bash block inside the section uses only `jq -e` assertions with no `mustmatch` directive, so the markdown spec collector silently skips it. Running `uv run --extra dev pytest spec/05-drug.md --collect-only -q | grep -i approval` returns only `Human-Friendly Approval Date (line 253)`; the adjacent `Compact Approval Fields (line 236)` block is not registered as a `BashItem`. The approval JSON contract is documented but unverified — a regression that drops any of those fields would slip through `make spec` and `make spec-pr` today.

Imported from March ticket 194. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/194-collect-compact-approval-fields-spec-block
