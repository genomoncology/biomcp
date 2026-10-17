---
flow: build
priority: 2
---

# 2139: Count full-text mentions in the variant identity check

Status: OPEN.

Milestone: 0.9.3

## Outcome

Variant article identity verification uses PubTator full-text annotations alongside the existing abstract evidence. The recorded replies confirm papers 34480199 and 30275001 under the existing identity rules. Paper 32371413 remains available and unverified, with a reason naming the missing typed Gene annotation.

Depends on: 2149
Depends on: 2156

- Starts from: `search_variant_articles_identity`, `verify_pubtator`, `PubTatorClient::export_biocjson` and the existing full-text implementation branch.
- Keeps: abstract confirmations, contradictions, expected-paper validation, deadlines, unverified answers without confirming evidence and main's repaired empty-autocomplete fixture.
- Changes: claims `sdlc/records/2139-identity-check-ignores-full-text-mentions-build.md` `src/entities/article/identity_verification.rs` `src/entities/article/variant_search.rs` `src/sources/pubtator.rs` `testdata/sources/pubtator/README.md` `testdata/sources/pubtator/export_fulltext_30275001.json` `testdata/sources/pubtator/export_fulltext_32371413.json` `testdata/sources/pubtator/export_fulltext_34480199.json` `testdata/sources/capture-receipts.json` `docs/user-guide/article.md` `docs/user-guide/variant.md` `skills/use-cases/17-exact-variant-literature.md` `tools/zero-coupling-historical.json` `CHANGELOG.md`.
- Proof: recorded full-text replies confirm papers 34480199 and 30275001 through `variant articles --verify-identity`; paper 32371413 stays available and unverified with a reason naming the missing typed Gene annotation. Unannotated, wrong-paper and wrong-alias cases cannot confirm; preserve existing wrong-gene contradiction behavior. Demonstrate the missing full-text behavior on current main, then run focused checks and the routine merged-candidate gate through build jobs.
- Defers: new identity trust rules.

PM decision: 2026-10-17. This adds an evidence source, not a new confirmation rule. Do not accept a new fallback based on a plain gene token unless a recorded owning decision authorizes it; preserve the existing identity rules or send the change for clinical review through the PM. Rebase without losing the repaired empty-autocomplete predicate. The provider pacing correction lands first.

## Clinical review of the existing rule

The clinical reviewer confirmed that `docs/user-guide/article.md`, `docs/user-guide/variant.md` and the earlier identity decisions require typed Gene/Variant annotations bound to the returned paper and matching gene identifiers. Same-passage co-occurrence cannot replace that evidence. The plain gene-token fallback on the adopted branch therefore changes the existing rule and has no authorization.

PM decision: 2026-10-17. The existing identity rule stands with no gene-token fallback. Close on the two supported confirmations and the available, unverified 32371413 candidate, whose reason names the missing typed Gene annotation.

Current-main job `1792241135767-38113` on `078ce756d` replayed the two focused regressions before the repair: the missing full-text request case failed and the unannotated guard passed, exit 100. Exports for the three recorded papers lacked `full=true`. The earlier compiler-wrapper failure with incremental compilation did not run the tests and is not product-red evidence.
