---
flow: build
priority: 5
---
# Decompose variant.rs into variant submodule

`src/entities/variant.rs` is 2,534 lines — a hot surface handling variant search, detail retrieval, population frequencies, and ClinVar/CIViC enrichment. After article/drug/disease/trial decompositions, variant is the next remaining god-file on the entity tier. Shrink it into `src/entities/variant/` following the established submodule pattern.

Completed under March on 2026-04-14, as March ticket 190. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/190-decompose-variant-rs-into-variant-submodule
