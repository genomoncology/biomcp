# Article keyword search has no relevance floor

Found in the botassembly knowledge-base run (2026-08-27,
experiments/188-shank3-knowledge-base, `kb/20-*` and `kb/24-*` captures):
the keyword query "SHANK3 modifier biomarker" returned materials-science
papers alongside real leads, and several other keyword searches mixed
off-topic rows into plausible ones. The synthesize stage had to flag the
noise manually in the knowledge base's cautions section.

This is the article-search sibling of the pathway scoping problem ticket
1063 just fixed (drop zero-relevance rows, surface partial-source loss):
article keyword search federates across sources with different relevance
models, and the merged result carries no floor. A relevance tier applied
at merge time — the same title/anchor-match discipline the pathway search
now has, or source-reported ranking retained instead of flattened — would
keep the card trustworthy for exactly the agents that cite it.

Recorded for triage; the experiment captures are the evidence.
