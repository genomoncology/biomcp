---
id: '018'
team: biomcp
name: Expand operator proof surfaces for new sources
status: ready
priority: 8
flow: build
repo: biomcp
dependencies: []
model-map: default
---

## Why

The source-addition checklist says new integrations should deliberately
evaluate `biomcp health` and `scripts/contract-smoke.sh`, but the KEGG, HPA,
ComplexPortal, and g:Profiler surfaces are still only partially represented in
operator checks. At the same time, several docs currently describe `health` as
a stronger readiness signal than the code actually provides.

## Scope

### In scope

- Decide which of KEGG, HPA, ComplexPortal, and g:Profiler belong in
  `biomcp health`, and implement or explicitly document the chosen scope.
- Extend `scripts/contract-smoke.sh`, targeted tests, or both so the newly
  shipped source surfaces have intentional operator proof.
- Align operator-facing docs with actual `health` scope and exit semantics.

### Out of scope

- New observability infrastructure beyond the existing health/smoke surfaces.
- Adding net-new data sources.

## Success Checklist

- [ ] Each newly shipped source is either covered by `biomcp health` or has a
      documented reason for staying out.
- [ ] `scripts/contract-smoke.sh` and/or targeted tests intentionally cover the
      new source surfaces that should have live contract probes.
- [ ] Operator-facing docs describe `health` and proof surfaces accurately.
- [ ] Release-facing verification guidance matches the chosen proof model.

## Notes

- Review artifacts: `.march/code-review.md`, `.march/architecture-review.md`
- Likely touchpoints: `src/cli/health.rs`, `scripts/contract-smoke.sh`,
  `design/technical/source-integration.md`, `docs/reference/data-sources.md`
