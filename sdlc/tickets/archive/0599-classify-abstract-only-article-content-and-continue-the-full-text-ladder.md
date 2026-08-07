---
flow: build
priority: 9
---
# Classify abstract-only article content and continue the full-text ladder

The full-text ladder treats any non-empty converted JATS as a winner even when the source contains only title and abstract. On current `main`, PMIDs 11805335 and 26951660 save four-line, 1,844-byte and 937-byte title/abstract files, report `source_kind: jats_xml` with `has_fulltext_signal: true`, and stop `--pdf` fallback. The current architecture promises that unusable XML falls through, but abstract-only XML is neither empty nor useful article body, so it escapes that rule.

Completed under March on 2026-07-20, as March ticket 599. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/599-classify-abstract-only-article-content-and-continue-the-full-text-ladder
