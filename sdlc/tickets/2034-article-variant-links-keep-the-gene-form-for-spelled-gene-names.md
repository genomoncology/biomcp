# 2034 — Article variant links keep the gene form for spelled gene names

Status: complete.

Milestone: 0.9.2

## Outcome

A variant row in `biomcp article entities <PMID>` that has a gene and a protein change prints the gene-plus-change command even when the article spells the gene "K-ras", "K-RAS" or another non-symbol form. A row never falls back to an rsID link that can open a different allele because of how the article spelled the gene.

## Evidence

Filed 2026-10-08 from the fourth independent review of main at `cf8113f3a` (ticket 2035, finding 3).

- `src/transform/article/annotations.rs:200-224` takes the gene's most-mentioned text as its symbol. `src/entities/variant/resolution.rs:259` builds the gene-plus-change form only when that text is all capitals and digits. "K-ras" and "K-RAS" fail that check, so the row falls back to the rsID.
- A probe test with "K-ras" twice, "KRAS" once and a single G12A row on `rs121913529` printed `rsID rs121913529`. That link opens G12D, the case 2029 set out to stop.
- `biomcp article entities 37887282`, which writes the gene as K-Ras and K-RAS, prints `get variant rs121913530` for G12C, and bare `get variant G12V` and `get variant G12D` for the others. G12C opens the right allele only because MyVariant ranks it first.
- The 2029 Outcome and the spec page do not mention this case.

- Starts from: ticket 2029 and its landing `8eec444db`.
- Keeps: 2029's gene-first rule; rsID links for rows with no gene; the mention-text fallback.
- Changes: choose the most-mentioned gene text that passes the symbol check; when a row has a gene identifier and no usable symbol text, build the gene form from the gene identifier's official symbol or the mention text, not the rsID.
- Proof: a test with "K-ras" mentions and a single G12A row prints `biomcp get variant "KRAS p.G12A"` and fails on `cf8113f3a`; a test covers a gene with no symbol-shaped mention.
- Defers: gene-less multi-allele rsID rows, which 2029 records as a known limit.

D
## Review record (2034)

- Code review: ACCEPT 2026-10-09 (fresh reviewer, full head). Spelled gene names keep the gene form; the official-symbol fallback verified with source evidence; report-only P2s recorded in the review artifacts (/tmp/review-2034-code.md).

## Root cause

`MutationContext::collect` (`src/transform/article/annotations.rs`) takes each
gene's most-mentioned text as its symbol without checking the text's shape, and
`gene_qualified_change` builds the gene-plus-change form only when that text
is an exact gene token — all capitals and digits after the first letter
(`is_exact_gene_token`, `src/entities/variant/resolution.rs`). An article that
spells the gene "K-ras" or "K-RAS" has a most-mentioned text that fails the
check, so the form never classifies as exact input and the row falls to the
document-based fallbacks: an rsID the document shows naming one change keeps
its link, and a multi-allele rsID with a protein change loses its identifier
entirely. The row's own data still carries the gene's NCBI identifier
(`gene_id` 3845 for the capture above) and the protein change, and MyGene
resolves 3845 to the official symbol KRAS, but nothing consults the
identifier when the mention text fails the shape check. The symbol-shaped
mention also loses to a more-mentioned spelled form: a document mentioning
"K-ras" twice and "KRAS" once picks "K-ras" today.

## Success criteria

- A document that mentions "K-ras" twice and "KRAS" once with a single G12A
  row on `rs121913529` prints `biomcp get variant "KRAS p.G12A"`: the
  most-mentioned gene text that passes the symbol check wins. A named test
  pins this and fails on `cf8113f3a`, where the row prints
  `biomcp get variant rs121913529`.
- A document whose every gene mention fails the symbol check ("K-Ras",
  "K-RAS") still prints the gene form when the gene identifier resolves to an
  official symbol: MyGene's batch gene lookup answers NCBI Gene 3845 with
  KRAS, so the G12C, G12V and G12D rows of the recorded PMID 37887282 capture
  print `biomcp get variant "KRAS p.G12C"`, `"KRAS p.G12V"` and
  `"KRAS p.G12D"` on the spec page. A named unit test pins the official-symbol
  path, and a test pins that a non-symbol official symbol builds no form.
- The official-symbol lookup runs only for gene identifiers that a mutation
  row carrying a protein change references and whose mentions are never
  symbol-shaped, so ordinary documents add no MyGene traffic. A failed lookup
  keeps 2029's fallbacks.
- 2029's keeps hold: rows with no gene keep their rsID links, rows with no
  usable form keep the mention-text command, and the existing named tests stay
  green.

## Build status

- Built on branch `tickets/2034-spelled-gene-names`, head `7fad90a9e`
  (`1c8d7b97b` plus the test-expectation fix and the receipts-digest repin),
  2026-10-08.
- Scoped verification on the build host: `cargo fmt --check` clean;
  `cargo clippy --no-default-features --lib --tests --locked -- -D warnings`
  clean; `cargo nextest run --no-default-features --locked -E
  'test(annotations) or test(mygene)'` 52 passed; `-E 'test(article) or
  test(pubtator)'` 712 passed; `make lint` green;
  `tools/check-source-capture-receipts.py --root testdata/sources` clean.
  After `make sync-python-dev` and `make prepare-spec`, `mustmatch test
  spec/entity/article-entities.md --lang bash --timeout 180` passed all 24
  blocks under the article and variant-identity fixtures. The new 37887282
  blocks print `biomcp get variant "KRAS p.G12C"`, `"KRAS p.G12V"` and
  `"KRAS p.G12D"`, and the fixture's request log records exactly one MyGene
  `ids=3845` batch per 37887282 command while 30738221 issues none.
- Red on main: with only
  `spelled_gene_mentions_pick_the_most_mentioned_symbol_shaped_text`
  appended over `origin/main` (`0b432cf0d`), the test fails with
  `namespace: "rsID"`, `identifier: "rs121913529"` — the link the ticket
  records opening G12D. It passes on this branch.
- Boundary: `get article`, `article entities` and batch article resolve
  official symbols through MyGene; the `variant articles` search enrichment
  (`resolve_variant_article_from_pmid`) keeps the mention-text rule, so a
  spelled-gene document's rows there still take the document-based fallbacks
  until that path grows deadline-aware MyGene support. The ticket's Outcome
  names `article entities` only.
- The receipts-manifest edit required repinning its digest in
  `tools/zero-coupling-historical.json`, following 2029's landing.
- Protocol note: one local `pytest tests/test_capture_receipts.py` run
  slipped during the lane (66 passed); the repo pre-commit hook also runs
  fmt and a dev-profile check on commit. Every other verification ran on the
  build host through `yr`.
