---
flow: build
priority: 8
---
# Complete article fulltext fallback ladder (PMC HTML + opt-in PDF)

Ticket 255 shipped the fulltext resolver boundary, source-aware cache key, dynamic labels, and `cargo deny` license gate. The actual fallback ladder still has two gaps: there is no HTML acquisition step between XML and the spike-proven PDF path, and the open-access PDF URL already surfaced by `semantic_scholar.open_access_pdf.url` is never consumed. Both gaps modify the same fallback dispatch in `src/entities/article/fulltext.rs`, share the 255 license gate, and need to be sequenced (`XML → PMC HTML → PDF (opt-in)`) rather than landed as two independent tickets that rewrite the same function in sequence.

Completed under March on 2026-04-20, as March ticket 256. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/256-add-opt-in-pdf-fallback-for-article-fulltext
