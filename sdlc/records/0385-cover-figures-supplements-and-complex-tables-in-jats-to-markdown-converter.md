---
base: ed8944c2da9161e528c0a4ff19ac8a4dd409379b
head: 487d79a3bc13c3f5103b476c6b2f7cd7d7f93c8b
---
BioMCP's JATS→Markdown converter silently drops content the source structure already carries, so an agent ingesting the saved Markdown believes it has the whole paper when it does not. On a real open-access article the converter dropped all four figure captions, rendered an empty "Supplementary Material" heading, and would drop a merged-cell table body with no trace — and those captions carried quantitative content ("n=10", "measurement bar is 70 μm", "significant reduction in FDG uptake"). This is the read-side of one principle: make coverage explicit; never silently drop or mangle content. The fix is pure rendering — no new network I/O.

Imported from March ticket 385. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/385-cover-figures-supplements-and-complex-tables-in-jats-to-markdown-converter
