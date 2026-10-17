# 2131 — Add LitVar2 variant literature to article search

Status: COMPLETE.

Milestone: 0.9.3

Reviews: revision ca7dea1, accept

## Outcome

`search article` with a variant (gene plus protein change, or rsID) also asks NCBI LitVar2 for the papers it links to that variant, and merges them with the existing sources, so variant papers PubTator and Europe PMC miss appear.

## Evidence

- Gap recorded on current main `43252305f` (macOS gate host, pm job `1792206115649-63202`, live providers, 2026-10-17). `biomcp --json variant articles rs372266620 --limit 50 --no-cache` exits 0 with resolution `ambiguous`, 0 results, and no sources, while LitVar2 links 39 papers to that rsID. `biomcp --json variant articles "CDKN2A A57V" --limit 50 --no-cache` exits 0 with 50 results from europepmc, pubmed, pubtator, and semanticscholar, but only 7 of LitVar2's 39 linked papers appear: 32 are missing (starting 10651484, 12752119, 14735200, 18023021, 19260062, 20876876, 21727090, 23243591, 24413734, 24728327). The gap shows on main.
- A literature-coverage test on 2026-10-16 found 27 CDKN2A papers through LitVar2 that BioMCP did not return. Main has no LitVar2 client or registry entry.
- LitVar2 is NCBI's free public variant-to-literature API, with no key. On 2026-10-16 its autocomplete answered `CDKN2A A57V` with rsID `rs372266620`, 39 papers, and `rs113488022` with BRAF p.V600E, 32,227 papers.
- The same test found no extra free copies through Unpaywall or OpenAlex, so this ticket adds LitVar2 only.

- Starts from: `src/sources/` (PubTator client as the model), `docs/reference/sources.json`, the article search fan-out in `src/entities/article/`.
- Decisions:
  - Resolve the variant through LitVar2 autocomplete, keep only an entry whose gene and change, or rsID, match the request exactly through the existing variant identity comparison, then fetch its publication list.
  - Merge LitVar2 PMIDs into the existing article results with source attribution, under the article search's existing deadline and per-source limits; a LitVar2 failure degrades that source only.
  - Register LitVar2 in the source registry with its terms stated, so `--disable-sources` and health cover it.
- Changes: claims `src/sources/litvar.rs` `src/sources/litvar/**` `src/sources/mod.rs` `src/entities/article/**` (search fan-out only) `src/error.rs` `src/sources/rate_limit.rs` `src/cli/health/**` `docs/reference/sources.json` `docs/sources/**` `docs/reference/data-sources.md` `docs/reference/source-licensing.md` `mkdocs.yml` `testdata/sources/litvar/**` `testdata/sources/capture-receipts.json` `tests/**` (source registry, source-page, and disable-walk contracts; package-file pin) `tools/zero-coupling-historical.json` `spec/fixtures/**` (the frozen variant-article fixtures answer the new route) `CHANGELOG.md`. The registry, health, licensing, source-page, replay-walk, and digest contracts each name the new source once it joins, so those files carry the one-line registration the contracts demand.
- Proof: recorded LitVar2 replies for `CDKN2A A57V` and `rs113488022`; a test shows LitVar2 papers merged into the results with attribution; a test shows a non-matching autocomplete entry is dropped; a LitVar2 error degrades only its source. One live run recorded in the review.
- Order: after 2130 and 2124 (which makes every registered source honor `--disable-sources`).
