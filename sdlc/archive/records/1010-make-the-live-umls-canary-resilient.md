---
flow: quickfix
priority: 9
---

# Make the live UMLS canary resilient

The repaired `make verify` reaches its credentialed UMLS discovery block, but the live assertion is stale against UMLS 2026AA. Searching `type 2 diabetes mellitus` now returns UMLS C0011860 under the preferred label `Diabetes Mellitus, Non-Insulin-Dependent`; the bounded result is filled by the exact MONDO concept and OLS prefix matches before that differently labelled UMLS row is selected for atom expansion. The product still contacts UMLS successfully, but the page incorrectly requires two exact code-source strings from the old result ordering.

Keep production discovery ranking, limits, provider code, and credential handling unchanged. In `spec/surface/discover-live.md`, use the current stable UMLS preferred label as the representative credentialed query, request bounded `--full` previews, and validate the parsed JSON shape rather than matching row order. Independently require the canonical `MONDO:0005148` concept. Require the exact `UMLS:C0011860` concept to have a UMLS source and to carry both a normalized SNOMED family xref and an ICD-10 family xref on that same concept. Accept version-specific family members such as `SNOMEDCT`/`SNOMEDCT_US` and `ICD10`/`ICD10CM` by uppercasing a string source and checking the explicit `SNOMEDCT` and `ICD10` prefixes; reject non-string or malformed source shapes. Do not let unrelated secondary UMLS rows satisfy the family checks, and do not weaken this to an `any code exists` check. A missing credential, provider error, malformed JSON, absent exact UMLS concept/source, or absent same-concept SNOMED or ICD-10 family must remain red.

Focused structure proof belongs in `tests/surface/test_parallel_isolation_contract.py`: the live page must keep the UMLS-only wrapper opt-in, request `--full`, parse JSON fail-closed, bind both source families to exact C0011860, reject malformed source shapes, and assert both independent identities without pinning result order. The corrected live page, focused structure test, `make verify`, `make lint`, and `git diff --check` must pass.
