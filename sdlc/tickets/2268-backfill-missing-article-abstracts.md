---
priority: 2
---

# 2268 — Backfill missing article abstracts

Status: OPEN.

Milestone: 0.9.4

## Outcome

Changes: `CHANGELOG.md`, `src/entities/article/enrichment.rs`, `src/entities/article/enrichment/abstracts.rs`, `src/sources/europepmc.rs`, `src/sources/pubmed.rs`, `src/transform/article/anchors.rs`, `src/transform/article/federation.rs`, `tests/test_article_abstract_recovery.py`, `tests/test_article_metadata_recovery.py`.

Article search rows under `--full` and `get article` carry the abstract whenever any queried source holds it.

## Evidence

Research ran released 0.9.3 on benchmark questions and paper fetches and verified each finding against the provider's official API; raw outputs are in research's store and may be copied as fixtures (ask the PM for the path). R7: `biomcp --json search article -g CXCR2 -k antagonists --type review --full --limit 5`: three rows found only by Europe PMC have no abstract_snippet, though Europe PMC resultType=core returns abstractText (998 and 1,602 characters for 36780833 and 42423773). R8: `search article -d glioma -k '"triple-negative gliomas"' --source pubmed --full` returns PMID 22890969 with no snippet; PubMed EFetch holds a 1,654-character abstract. R9: `biomcp --json get article 41790359` has no abstract_text; PubTator3 biocjson has a zero-length abstract passage, while PubMed EFetch and Europe PMC core hold it.

## Keeps

Treat an empty abstract as missing and fall back. Stay within the request deadline; batch backfill requests.

## Proof

Replay tests for all three cases show the abstract present with its source.

## Implementation evidence

Fresh developer completed f33558cfd4dba88debc3d522c48bd75c7a17f3ff. Current main failed all four public cases; accepted 2273 failed three, while get 41790359 already passed through 2273. Final M5 job 2268/1792439264613-19661 passed nine CLI controls, 64 affected Rust tests, formatting, Ruff, warnings-denied Clippy and quality checks. Fresh reviewer accepted that exact revision and independently matched every changed source and test with the checked files; no redundant tests ran.

Europe PMC requests core results and retains nested journal metadata. Visible PubMed rows recover missing abstracts in bounded batches through the shared citation parser and existing search deadline. Cleanup preserves literal comparisons and decimal characters. The visible_metadata, backends and search modules remain unchanged. Private direct-provider replies confirmed the missing-lite/core abstract and journal shapes; only synthetic regressions are public. No full landing gate is claimed.

Keep: experiments/2090-biomcp-2268-abstract-recovery — retain actual replies, research outputs, patches, hashes and red/green job logs.
