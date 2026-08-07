---
base: 0fd32adcec3653d34f3c8b18281c7191b92e55c3
head: 0a7e7f71e549ff1a5d6596bb2d1077d79d13bcac
---
Ticket 255 shipped the fulltext resolver boundary, source-aware cache key, dynamic labels, and `cargo deny` license gate. The actual fallback ladder still has two gaps: there is no HTML acquisition step between XML and the spike-proven PDF path, and the open-access PDF URL already surfaced by `semantic_scholar.open_access_pdf.url` is never consumed. Both gaps modify the same fallback dispatch in `src/entities/article/fulltext.rs`, share the 255 license gate, and need to be sequenced (`XML → PMC HTML → PDF (opt-in)`) rather than landed as two independent tickets that rewrite the same function in sequence.

Imported from March ticket 256. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/256-add-opt-in-pdf-fallback-for-article-fulltext
