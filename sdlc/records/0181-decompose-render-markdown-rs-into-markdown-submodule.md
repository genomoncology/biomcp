---
base: 469ae4147dc5db2d978515d16a1491eb03b6b292
head: 16f6ef3ad1c1e5e07856659393bca6fa9f6b58d0
---
`src/render/markdown.rs` is 10,969 lines — the second-largest file in the codebase. It renders every entity type in the system (gene, article, disease, drug, variant, trial, PGx, pathway, protein, adverse event, study, discovery) plus shared formatting, pagination, and URL generation. A contributor looking at drug rendering has to scroll past 2,000 lines of other entity renderers. The test block alone is 5,941 lines with 135 test functions.

Imported from March ticket 181. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/181-decompose-render-markdown-rs-into-markdown-submodule
