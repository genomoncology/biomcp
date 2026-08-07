---
flow: build
priority: 5
---
# Cover figures, supplements, and complex tables in JATS-to-Markdown converter

BioMCP's JATS→Markdown converter silently drops content the source structure already carries, so an agent ingesting the saved Markdown believes it has the whole paper when it does not. On a real open-access article the converter dropped all four figure captions, rendered an empty "Supplementary Material" heading, and would drop a merged-cell table body with no trace — and those captions carried quantitative content ("n=10", "measurement bar is 70 μm", "significant reduction in FDG uptake"). This is the read-side of one principle: make coverage explicit; never silently drop or mangle content. The fix is pure rendering — no new network I/O.

Completed under March on 2026-06-03, as March ticket 385. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/385-cover-figures-supplements-and-complex-tables-in-jats-to-markdown-converter
