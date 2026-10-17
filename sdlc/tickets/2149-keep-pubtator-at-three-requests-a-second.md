---
flow: build
priority: 2
---

# 2149: Keep PubTator at three requests a second

Status: OPEN.

Milestone: 0.9.3

## Outcome

PubTator requests obey the documented three-per-second budget with or without an NCBI key; the keyed ten-per-second budget remains with E-utilities.

Depends on: 2156

- Starts from: `pubtator_min_interval`, its existing interval tests and the existing implementation branch.
- Keeps: keyed E-utilities and all other provider policies.
- Changes: claims `src/sources/rate_limit.rs` `docs/blog/biomcp-pubmed-articles.md` `docs/getting-started/api-keys.md` `docs/reference/configuration.md` `docs/reference/data-sources.md` `docs/reference/sources.json` `tests/test_source_licensing_docs_contract.py` `CHANGELOG.md`.
- Proof: verify the provider's current primary documentation and retain its citation. The existing interval cases show PubTator at 334 ms for both key states and E-utilities at 334/100 ms without/with the key; their corrected expectation fails on current main before implementation. Run affected checks and the routine merged-candidate gate through build jobs.
- Defers: other rate-limit policies.

PM decision: 2026-10-17. Reuse the existing narrow correction after validating its source citation and evidence. This precedes the added full-text annotation traffic.
