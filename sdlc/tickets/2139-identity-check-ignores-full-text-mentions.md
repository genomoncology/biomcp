---
flow: build
priority: 2
---

# 2139: Count full-text mentions in the variant identity check

Status: OPEN.

Milestone: 0.9.3

## Outcome

Variant article identity verification uses PubTator full-text annotations alongside the existing abstract evidence and confirms the three recorded SAMHD1 I448T papers under the existing identity rules.

Depends on: 2149
Depends on: 2156

- Starts from: `search_variant_articles_identity`, `verify_pubtator`, `PubTatorClient::export_biocjson` and the existing full-text implementation branch.
- Keeps: abstract confirmations, contradictions, expected-paper validation, deadlines, unverified answers without confirming evidence and main's repaired empty-autocomplete fixture.
- Changes: claims `src/entities/article/identity_verification.rs` `src/entities/article/variant_search.rs` `src/sources/pubtator.rs` `testdata/sources/pubtator/**` `testdata/sources/capture-receipts.json` `docs/user-guide/article.md` `docs/user-guide/variant.md` `skills/use-cases/17-exact-variant-literature.md` `tools/zero-coupling-historical.json` `CHANGELOG.md`. Narrow capture paths to the actual three recordings and README before implementation.
- Proof: recorded full-text replies confirm papers 32371413, 34480199 and 30275001 through `variant articles --verify-identity`; unannotated, wrong-paper, wrong-gene-token and wrong-alias cases stay unverified. Demonstrate the missing full-text behavior on current main, then run focused checks and the routine merged-candidate gate through build jobs.
- Defers: new identity trust rules.

PM decision: 2026-10-17. This adds an evidence source, not a new confirmation rule. Do not accept a new fallback based on a plain gene token unless a recorded owning decision authorizes it; preserve the existing identity rules or send the change for clinical review through the PM. Rebase without losing the repaired empty-autocomplete predicate. The provider pacing correction lands first.
