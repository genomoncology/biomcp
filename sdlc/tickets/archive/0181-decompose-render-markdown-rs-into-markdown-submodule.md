---
flow: build
priority: 5
---
# Decompose render/markdown.rs into markdown submodule

`src/render/markdown.rs` is 10,969 lines — the second-largest file in the codebase. It renders every entity type in the system (gene, article, disease, drug, variant, trial, PGx, pathway, protein, adverse event, study, discovery) plus shared formatting, pagination, and URL generation. A contributor looking at drug rendering has to scroll past 2,000 lines of other entity renderers. The test block alone is 5,941 lines with 135 test functions.

Completed under March on 2026-04-12, as March ticket 181. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/181-decompose-render-markdown-rs-into-markdown-submodule
