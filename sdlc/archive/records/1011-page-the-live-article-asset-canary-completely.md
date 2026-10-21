---
flow: quickfix
priority: 9
---

# Page the live article asset canary completely

The final `make verify` run at `1b825668` failed the PMID 20516115 supplement check even though a fresh no-cache query, current NCBI JATS, and current PMC HTML all still expose both named supplements. The live page uses the compact asset view, which returns only the first 10 coverage rows; the provider now yields 14 rows. The two required files currently happen to be rows 6 and 7, so upstream row additions or transient route availability can push correct results outside the assertion's page.

Keep production article discovery, sorting, retrieval, proof-of-work handling, and limits unchanged. In `spec/entity/article-assets-live.md`, request both bounded complete views—`coverage` and `retrievable`—with a limit large enough to contain the current manifest, and require both responses to say there is no continuation. The views must come from the same cached manifest. Find both named suffixes without depending on row order. For each file, accept exactly one of these states:

- a retrievable asset with provider-labelled discovery routes from both `jats_xml` and `pmc_html`; or
- a coverage row with outcome `pmc_proof_of_work` or `source_unavailable`, both discovery routes, no handle, and no matching retrievable asset.

Repeated uncached review showed that PMC can transiently return `source_unavailable` for one of the two recognized links even while both source documents still advertise it. Do not accept `healthy_absent`, `unsupported_origin`, a generic package miss, missing named coverage, or a row appearing in both states, and do not claim challenge HTML is a binary. Keep byte, hash, media-type, and stable-handle proof for retrievable assets in the existing deterministic routine contract rather than making the live canary depend on mutable download policy.

Focused proof belongs in `tests/surface/test_parallel_isolation_contract.py`: require both explicit views and bounds, complete pagination in both responses, same-cache execution, both filenames, both discovery routes, the mutually exclusive state rules, and order-independent parsing. Missing files, partial pagination, malformed rows, missing route provenance, duplicate state, or any other outcome must remain red. The focused test, the live page, `make verify`, `make release-gate`, `make lint`, and `git diff --check` must pass.
