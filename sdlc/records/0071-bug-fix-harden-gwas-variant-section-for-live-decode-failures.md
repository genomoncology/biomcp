---
base: 257f7a8d82dd8e41831bf2591150d2825939d1d5
head: 4f1155e2c57b7c3817c64e436964274dbd26cd6f
---
`make spec-pr` currently fails the `spec/03-variant.md::GWAS Supporting PMIDs` heading. The underlying command `biomcp --json get variant rs7903146 gwas` fails with `Error: HTTP request failed: error decoding response body`. The GWAS REST API is returning a response that cannot be decoded by the current deserializer.

Imported from March ticket 071. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/071-bug-fix-harden-gwas-variant-section-for-live-decode-failures
