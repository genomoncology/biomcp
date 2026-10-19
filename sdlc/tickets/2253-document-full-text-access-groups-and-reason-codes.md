---
priority: 2
---

# 2253 — Document full-text access groups and reason codes

Status: OPEN.

Milestone: 0.9.4

## Outcome

Users and agents can tell from the docs which papers BioMCP can read in full, which only `--pdf` may reach, and which it cannot reach, and every full-text reason code is explained. Examples use papers that work.

## Evidence

Research audit of 0.9.3 docs:
1. `docs/reference/data-sources.md` near line 197 gives the full-text ladder order but not the three groups: papers with a PMC copy (including author manuscripts and read-only PMC) read through Europe PMC XML, EFetch or PMC HTML; papers free only on the publisher's site, reachable only by `--pdf` and often refused by bot protection; paywalled papers without a PMC copy, not reachable. Nothing says PubTator, Europe PMC and LitVar2 hold full text only for PMC papers, or that BioMCP never bypasses bot protection for full text.
2. `docs/user-guide/article.md` near lines 394-397 documents 6 of 13 reason codes; `body_detected`, `abstract_without_body`, `metadata_without_body`, `no_content`, `unusable_content`, `bot_challenge` and `source_unavailable` are missing.
3. No source pages exist for Europe PMC, PMC or PubTator.
4. Full-text examples in help, `cli-reference.md` near line 579 and `docs/how-to/find-articles.md` use 22663011, which is not in PMC and exits 1; 24200969 works.
5. `README.md` lines 25-26 say search article fans out across PubTator3 and Europe PMC, while line 322 and the docs include PubMed and Semantic Scholar.

## Keeps

Describe behavior as the code does it; read the code for each claim. Add the agent skill note beside the user-guide section.

## Proof

A docs check or test lists every reason code in the code and finds each documented. Every changed example runs successfully offline from fixtures or is marked live.
