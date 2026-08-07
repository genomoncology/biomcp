---
base: 2605a538f5ec3102cee02e1a15cb89e164fba6db
head: 2fc984825981401ac08c5413c87c914d0466180c
---
The full-text ladder treats any non-empty converted JATS as a winner even when the source contains only title and abstract. On current `main`, PMIDs 11805335 and 26951660 save four-line, 1,844-byte and 937-byte title/abstract files, report `source_kind: jats_xml` with `has_fulltext_signal: true`, and stop `--pdf` fallback. The current architecture promises that unusable XML falls through, but abstract-only XML is neither empty nor useful article body, so it escapes that rule.

Imported from March ticket 599. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/599-classify-abstract-only-article-content-and-continue-the-full-text-ladder
