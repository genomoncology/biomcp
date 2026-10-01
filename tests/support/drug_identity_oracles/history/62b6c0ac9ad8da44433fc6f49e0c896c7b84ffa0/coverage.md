# Alternative coverage

156 identified active static alternatives cover the 40 product requirements below. META-01 and META-02 remain in metadata.json. Historical rejected objects are excluded from active counts. No execution claim follows.

| Requirement | Named alternatives |
| --- | --- |
| P1-01 | `get-profile`, `search-profile-size-offset` |
| P1-02 | `empty`, `trimmed`, `query-1024`, `query-1025`, `limit-0`, `limit-1`, `limit-50`, `limit-51`, `window-inclusive`, `window-offset`, `window-over`, `name-256`, `name-257`, `escaping`, `empty-name` |
| P1-03 | `non-success`, `wrong-content-type`, `body-over-limit`, `valid` |
| P1-04 | `ordered-escaped-bytes`, `two-ordinals`, `nonselected-invalid` |
| P1-05 | `u64-0`, `u64-18446744073709551615`, `u64-over-usize32`, `missing-total`, `wrong-total` |
| P2-01 | `display-ndc`, `display-generic`, `display-brand`, `display-drugbank`, `display-chembl`, `display-gtopdb`, `display-unii`, `display-chebi`, `all-eight-first-elements` |
| P2-02 | `trim-dots-case`, `dots-only`, `nameless` |
| P2-03 | `salt`, `brand`, `no-match-stable-tie-conflicting-ids`, `adopted-salt`, `adopted-brand` |
| P2-04 | `mechanism-target`, `duplicates-nameless-truncate-provider-total`, `duplicates-and-nameless-before-limit`, `adopted-target-mechanism-filter` |
| P2-05 | `later-exact-duplicate-upgrade-pages-offset`, `window-total-over` |
| P2-06 | `empty-first-page-unstructured`, `later-page`, `structured`, `nonempty`, `local-fallback-total-and-order` |
| P2-07 | `empty`, `same`, `two`, `one-different` |
| P2-08 | `label-canonical`, `unique-discover`, `ambiguous-alias` |
| P3-01 | `repeated-equal-conflicting-two-rows`, `repeated-equal-code-origins`, `retained-strict-invalid-document` |
| P3-02 | `missing-null-empty-array`, `object-array-and-scalar-array`, `retained-exhaustive-shapes` |
| P3-03 | `actual-normalization-dedupe-cap`, `first-code-conflict`, `discard-search-row`, `fallback` |
| P3-04 | `all-source-only-enrichment`, `invalid-identity-companion` |
| P4-01 | `nonmatching-invalid`, `blank-scalar`, `mixed-blank-array`, `wrong-admitted-type` |
| P4-02 | `duplicate`, `malformed-envelope`, `byte-one-over`, `byte-inclusive` |
| P4-03 | `malformed-source-only` |
| P4-04 | `fallback-search`, `alternative-detail` |
| P4-05 | `label-detail`, `discover-detail` |
| P4-06 | `eu-identity-catch`, `who-identity-catch`, `all-identity-catch` |
| P4-07 | `first`, `repeat` |
| P4-08 | `notfound`, `connection`, `timeout`, `generic-api`, `generic-unavailable` |
| P4-09 | `all-failure-channels` |
| P5-01 | `explicit-approvals-json-markdown`, `default-targets`, `default-json-markdown-adopted` |
| P5-02 | `search-order-count-pagination`, `batch-order`, `empty-region-buckets`, `regional-order-envelopes`, `search-markdown`, `raw-mcp-search-json`, `raw-mcp-search-markdown` |
| P5-03 | `not-requested-all-explicit`, `empty-data-unavailable-approvals`, `partial-interactions`, `required-label-zero-ddinter`, `raw-region-restrictions` |
| P5-04 | `raw-get-json`, `typed-get-json`, `raw-get-text`, `rejected-raw-and-typed` |
| P5-05 | `get-region`, `search-drug` |
| P5-06 | `typed-adopted-alias-search`, `typed-opt-out-refusal`, `raw-opt-out` |
| P5-07 | `vaccine-bypass`, `unknown-total`, `exhausted`, `api-filter`, `finished-filter` |
| P6-01 | `anchor-card3-ddinter32-combination` |
| P6-02 | `matching-conflicting`, `anchorless`, `absent-id-neutral`, `later-unii-conflict`, `adopted-all-unii-conflict` |
| P6-03 | `requested-distinct-canonical-three-provider`, `accepted-literal-five-alias`, `eligible-simple-code-unicode-apostrophe`, `excluded-systematic-long-descriptor`, `canonical-requested-collision`, `requested-only-absence` |
| P6-04 | `normalized-key-refresh`, `transient-uncached`, `terminal-uncached` |
| P6-05 | `union-dedup-exact-total`, `continuation-unknown`, `provenance`, `adopted-empty-union-count`, `opt-out` |
| P6-06 | `every-allowed-field`, `excluded-nonexact-nohit`, `field-order-labels`, `adopted-source-exact` |
| P6-07 | `adopted-civic-display`, `protected-target-family-variant-indication`, `opt-in-pharmacodb` |
