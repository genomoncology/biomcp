---
base: 9dd70c9767c25a4171fa47d0052f12c3fb50a20b
head: 4210583606e60844ed3506d4567124350d7b55bd
---
`src/render/markdown/related/article_support.rs:8` hardcodes `const ARTICLE_DRUG_ALLOWLIST: &[&str] = &["psoralen"];` — a single drug token hand-added because it does not match the suffix-based drug heuristics (`ARTICLE_DRUG_SUFFIXES`: `mab`, `nib`, `statin`, …). It is used in `first_article_drug_token` (`article_support.rs:180`) to force-classify the literal string "psoralen" as a drug when deciding whether to append a cross-entity pivot **suggestion hint** to related-article output.

Imported from March ticket 486. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/486-remove-hardcoded-article-drug-allowlist-psoralen-token-from-article-pivot-hints
