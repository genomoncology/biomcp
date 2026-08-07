---
flow: build
priority: 5
---
# Decompose drug.rs into drug submodule

`src/entities/drug.rs` is 3,904 lines handling seven concerns: type definitions, query building, search, label parsing/extraction, FDA metadata processing, drug resolution/lookup, and regional identity builders — plus 1,015 lines of tests. Navigating the file requires scrolling past 1,500 lines of label extraction to find the public API.

Completed under March on 2026-04-11, as March ticket 178. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/178-decompose-drug-rs-into-drug-submodule
