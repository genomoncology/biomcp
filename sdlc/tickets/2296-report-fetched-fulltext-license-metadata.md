---
priority: 2
---

# 2296 — Report fetched fulltext license metadata

Status: OPEN.

Milestone: 0.9.4

Depends on: 2294

## Outcome

When a fetched fulltext source’s own metadata states a license, get article fulltext reports that license with truthful source attribution. Unknown is allowed only when no fetched source states one. Reuse the existing license classification and restrictions rather than adding a new licensing policy.

Changes: `src/entities/article/fulltext.rs`, `src/sources/pmc_oa.rs`, `src/sources/pmc_oa/tests/*`, `tests/test_article_fulltext_license.py`, `tests/golden/cases/get-article-23056577-fulltext.*`, `sdlc/records/2296-fulltext-license-metadata.md`, `CHANGELOG.md`. Narrow or correct these claims after locating the source of the lost fact.

## Evidence and PM ruling

Accepted recorded-command coverage2236 observes get article23056577 fulltext returning license_present:false and reuse unknown, although the fetched current PMC3467229.1 metadata explicitly records CC BY and is_pmc_openaccess:true. Its exact captured metadata SHA256 is4240e488862f7ed12db13a2a12c162102dd54e54f18df9e156436bc3779acb6b. The PM classified this as a required0.9.4 product bug on2026-10-20. A coverage disclosure is not its disposition. Keep2236 scope unchanged.

## Proof

Before code, restate the outcome, reproduce on current main or the approved dependency candidate, name the exact command/output and files, and add a meaningful outside-in failure using the already retained PMC3467229 captures. Show the final real CLI JSON/Markdown reports the supplied CC BY fact; preserve unknown when no source supplies a license and existing restricted/no-access behavior. Do not infer permission from an OA flag alone or weaken source restrictions. If sources conflict in a way existing rules do not resolve, ask the PM rather than inventing policy.

Run only affected checks through pm job. Reuse captured replies; no new capture unless a genuinely needed field is absent, and then only under the standing approved public-keyless rule. Existing body, sections, references and asset bytes remain outside the fix unless the actual propagation path requires them. Update only affected recorded output and preserve exact source bindings. No full suite or new live collection during development.

Keep: experiments/2102-biomcp-2258-final-live-replay — preserve the observed metadata and prior fulltext output while this repair and final admission consume them. Name the repair’s raw job folder in its record.
