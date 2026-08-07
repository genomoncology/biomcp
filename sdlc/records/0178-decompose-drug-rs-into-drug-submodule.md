---
base: ecdbf9a1aee80bbc686aa44ddc3701e00f8b939b
head: 026584d837f5bf554084d38132a1152b4a45f6de
---
`src/entities/drug.rs` is 3,904 lines handling seven concerns: type definitions, query building, search, label parsing/extraction, FDA metadata processing, drug resolution/lookup, and regional identity builders — plus 1,015 lines of tests. Navigating the file requires scrolling past 1,500 lines of label extraction to find the public API.

Imported from March ticket 178. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/178-decompose-drug-rs-into-drug-submodule
