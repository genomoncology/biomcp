---
priority: 2
---

# 2268 — Backfill missing article abstracts

Status: OPEN.

Milestone: 0.9.4

## Outcome

Article search rows under `--full` and `get article` carry the abstract whenever any queried source holds it.

## Evidence

Research ran released 0.9.3 on benchmark questions and paper fetches and verified each finding against the provider's official API; raw outputs are in research's store and may be copied as fixtures (ask the PM for the path). R7: `biomcp --json search article -g CXCR2 -k antagonists --type review --full --limit 5`: three rows found only by Europe PMC have no abstract_snippet, though Europe PMC resultType=core returns abstractText (998 and 1,602 characters for 36780833 and 42423773). R8: `search article -d glioma -k '"triple-negative gliomas"' --source pubmed --full` returns PMID 22890969 with no snippet; PubMed EFetch holds a 1,654-character abstract. R9: `biomcp --json get article 41790359` has no abstract_text; PubTator3 biocjson has a zero-length abstract passage, while PubMed EFetch and Europe PMC core hold it.

## Keeps

Treat an empty abstract as missing and fall back. Stay within the request deadline; batch backfill requests.

## Proof

Replay tests for all three cases show the abstract present with its source.
