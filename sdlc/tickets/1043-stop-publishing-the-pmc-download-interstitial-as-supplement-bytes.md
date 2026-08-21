---
flow: quickfix
priority: 19
---
# Stop publishing the PMC download interstitial as supplement bytes

The article asset surface advertises a PMC-linked supplementary file and then hands back PMC's anti-scraping challenge page instead of the file. The bytes are HTML, they are labelled with the supplement's media type, and nothing tells the caller that what they received is not the document they asked for.

## Reproducing it

```
biomcp --json get article PMC7857465 assets
```

The manifest contains two entries whose `filename` is `TBBE_A_1426496_SM7925.docx`. One comes from Europe PMC and is 837,946 bytes — that one is the real document. The other looks like this:

```json
{
  "filename": "TBBE_A_1426496_SM7925.docx",
  "asset_key": "pmc-29de9cb1207e7fb57003b99bb6c9915730c3a3dd12d4bb06992a27c39146b66c--TBBE_A_1426496_SM7925.docx",
  "kind": "supplementary-file",
  "media_type": "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
  "size_bytes": 1817,
  "provider": { "label": "PMC Linked Article Asset", "source": "PMC" }
}
```

Running that entry's own `handle` returns roughly 1,814 bytes of HTML whose title is `Preparing to download ...` and whose body contains:

```
const POW_CHALLENGE = "VwR3BQpmZmDjBQRhBQL2AGtv:..."
const POW_COOKIE_NAME = "cloudpmc-viewer-pow"
```

The exact byte count and the recorded `sha256` move between calls, because the challenge string is generated per request. Any 1.8 KB entry from the `PMC Linked Article Asset` provider on this article is the same interstitial.

## Why this is a defect and not a missing feature

BioMCP already recognises this page. `is_pmc_proof_of_work` in `src/sources/pmc_article.rs:233` looks for the markers `cloudpmc-viewer-pow` and `POW_CHALLENGE`, and the captured page contains both of them. `fetch_with_limit` in the same file checks that marker before it will return `PmcLinkedFetch::Bytes`, and `src/entities/article/assets.rs:847` maps a `ProofOfWork` result to `ArticleAssetNamedOutcome::PmcProofOfWork`, which is coverage with no asset key and no handle. `architecture/functional/article-fulltext.md` states the intended contract in the same words: "PMC proof-of-work responses remain named `pmc_proof_of_work` coverage with no asset key or handle."

So the guard, the outcome, and the documented contract are all already in place, and the interstitial is reaching a caller anyway. Find where the served bytes get past the marker check and close that path.

## The constraint that decides the shape of the fix

The marker check must hold for the bytes that are actually published, whichever route produced them. Do not make the answer depend on the declared `Content-Type`: PMC serves this page under the supplement's own media type, so a content-type test cannot see it.

## Done when

- No asset entry for `PMC7857465` from the `PMC Linked Article Asset` provider advertises `TBBE_A_1426496_SM7925.docx`, and the file appears in coverage as `pmc_proof_of_work` with no `asset_key` and no `handle`.
- The Europe PMC copy of that same filename is still advertised and still retrievable, unchanged.
- A page carrying either marker never leaves the process as asset bytes, regardless of the media type the provider declared for it.
- The proof runs from a stored copy of the interstitial, not from a live call. A copy is already checked in at `testdata/sources/pmc_article/pmc3040717-supplementary-tables-pow.html` and can be reused.

## One difference worth checking first

`recorded_pow_interstitial_is_not_returned_as_bytes` serves that fixture with `Content-Type: text/html; charset=utf-8`. In the live case PMC serves the same page under the supplement's own media type — `application/vnd.openxmlformats-officedocument.wordprocessingml.document` — which is what the manifest ends up recording. The existing test therefore never exercises the shape that is leaking. That is a lead, not a conclusion; confirm the actual cause before changing anything.

## Existing tests that pin this

`src/sources/pmc_article.rs` already holds the unit test `pow_markers_are_case_insensitive` (line 614) and the recorded-fetch tests `recorded_pow_interstitial_is_not_returned_as_bytes` (line 622) and `proof_of_work_is_retained_when_a_later_linked_target_fails` (line 694). All of these assert the behavior this ticket wants and all of them pass today, which is why the leak is somewhere else. They are correct as written.

`src/entities/article/assets.rs` asserts the `ncbi_interstitial` reason string at line 2597. That is also correct as written.

No shipped test asserts that the interstitial may be published as an asset. No test restatement is needed or authorized. Add coverage; do not weaken any assertion listed above.
