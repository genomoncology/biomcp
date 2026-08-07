---
base: a16806f1b95a040f95746d308135b971f3586cfc
head: 8157da1f53fe90563b41de297937e2bffb132a3e
---
The architecture review found next-command ownership leaking across layers: some entity code imports markdown quoting helpers to construct semantic guidance, while JSON envelopes are built unevenly across dispatchers. This makes shell safety and JSON follow-up behavior inconsistent and hard to ratchet.

Imported from March ticket 469. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/469-separate-typed-next-command-ownership-from-markdown-shell-quoting
