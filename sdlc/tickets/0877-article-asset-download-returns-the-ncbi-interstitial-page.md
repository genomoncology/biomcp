---
flow: build
priority: 6
---
# Refuse an article asset that arrives as NCBI's interstitial page

## Narrowed 2026-08-09 during review triage

The byte refusal and the manifest proof already landed as ticket 0679
(`sdlc/records/0679-reject-the-pmc-proof-of-work-interstitial-instead-of-
publishing-it-as-supplement-bytes.md`): `is_pmc_proof_of_work` classifies
the challenge page and `PmcLinkedFetch::ProofOfWork` survives source
classification without publishing bytes (src/sources/pmc_article.rs).
What remains of this ticket is ONLY the direct-retrieval error
reporting: the "Done when" below is reduced to the typed error. Do not
re-implement the detection or the manifest handling; build the error on
the landed classification.

## Done when

`biomcp --json get article 30311380 asset "NIHMS987696-supplement-Supp_Tables.xlsx"`
returns a typed error naming the interstitial and pointing at the PMC
URL a browser can complete the challenge at, instead of a generic
failure. (The bytes are already refused and the manifest already marks
the asset as a coverage outcome — landed in 0679.)

## The finding

Raised as an issue during BioMCP research on 2026-08-08, then folded
in here and the issue file removed. Reproduced in full below.

<!-- from article-asset-download-returns-the-ncbi-interstitial-page.md -->

# Downloading an article asset can return NCBI's interstitial page as the asset

Severity: should-fix. Wrong bytes handed over as if they were right.

    biomcp --json get article 30311380 asset \
      "NIHMS987696-supplement-Supp_Tables.xlsx"

returns HTML, not a spreadsheet:

    <html><head>…<title>Preparing to download ...</title>…
    <h1>Preparing to download ...</h1>
    <p id="discl"><a href="https://www.hhs.gov/vulnerability-disclosure-policy/…">
    HHS Vulnerability Disclosure</a></p>
    <script type="module" … src="https://cdn.ncbi.nlm.nih.gov/pmc/…/pow-o51sQKbL.js">

That is NCBI's JavaScript proof-of-work interstitial. A browser runs
the script and is then handed the file; a plain HTTP client is
handed this page with a 200. BioMCP passes it through as the asset,
so a caller that writes the result to `Supp_Tables.xlsx` gets a file
that opens as garbage, with nothing anywhere saying so.

The manifest already knows. `get article 30311380 assets` reports:

    "filename":   "NIHMS987696-supplement-Supp_Tables.xlsx",
    "media_type": "text/html",
    "size_bytes": 1817

An `.xlsx` that is `text/html` and 1,817 bytes is the interstitial,
every time. The signal is captured and then not acted on.

Fix shape, cheapest first:

- **Refuse rather than mislead.** When the retrieved bytes are HTML
  and the asset is not an HTML asset, return a typed error naming
  the interstitial and pointing at the article's PMC URL. A caller
  who knows they got nothing is far better off than one who thinks
  they got a spreadsheet.
- **Flag it in the manifest too.** A `media_type` that contradicts
  the filename extension should surface as a coverage outcome
  (`unsupported_origin` and friends already exist for this), not as
  a normal asset with a working-looking `handle`.
- Whether the interstitial can be satisfied at all is a separate
  question and probably not worth chasing. Refusing honestly closes
  this issue on its own.

Note this is the same article whose Europe PMC route is covered by
`europe-pmc-not-open-access-is-reported-as-a-failure.md`. Different
defect, same run: one source said "absent" and was recorded as
broken, another said "here it is" and handed over a placeholder.
Both push the caller toward believing something untrue about
supplementary material.

Found 2026-08-08 while researching PTEN GN003 for varclassify2.
