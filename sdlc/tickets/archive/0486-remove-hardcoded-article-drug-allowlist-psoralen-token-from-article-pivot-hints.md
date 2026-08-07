---
flow: quickfix
priority: 5
---
# Remove hardcoded ARTICLE_DRUG_ALLOWLIST psoralen token from article pivot hints

`src/render/markdown/related/article_support.rs:8` hardcodes `const ARTICLE_DRUG_ALLOWLIST: &[&str] = &["psoralen"];` — a single drug token hand-added because it does not match the suffix-based drug heuristics (`ARTICLE_DRUG_SUFFIXES`: `mab`, `nib`, `statin`, …). It is used in `first_article_drug_token` (`article_support.rs:180`) to force-classify the literal string "psoralen" as a drug when deciding whether to append a cross-entity pivot **suggestion hint** to related-article output.

Completed under March on 2026-07-09, as March ticket 486. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/486-remove-hardcoded-article-drug-allowlist-psoralen-token-from-article-pivot-hints
