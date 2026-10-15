---
flow: build
priority: 2
deps: [2041]
---

# 2097: Port the 0.9 drug identity work into the 1.0 resolver

Status: OPEN.

Milestone: 1.0

## Evidence

- Changes: `src/entities/drug/get/resolver.rs` and the resolver's custody model; main's identity stages and their test modules return to the tree here.
- Current facts: the 2041 merge keeps the 1.0 resolver structure and main's composable layer, and excludes main's 2050/2090 pinning test modules (transform/drug/name_resolution_tests.rs, test_support/{label_choice,ingredient_resolution,brand_resolution,product_names}.rs, label/tests/identity.rs, get/tests.rs additions) because the resolver does not implement that behavior. Until this ticket lands, the 1.0 line lacks 0.9's label-identity voting and ambiguous-shared-name refusal.
- Work: port the label-identity voting, ambiguous-name refusal, and bounded discover rescue into the resolver so each new identity stage emits its custody events; restore every excluded test module and make it pass; the parallel card fetch (2090) already survives the merge through the thin resolver-backed bridge.

## Acceptance

1. Every test module excluded by 2041 exists again and passes unmodified.
2. A consumer test shows a custody report carrying the identity stages' events.
3. An ambiguous shared drug name still refuses with candidates on the 1.0 line, proven by the restored 2085-shaped test.

## Exclusions

No changes to 0.9 main behavior; this ticket only touches the 1.0 line.
