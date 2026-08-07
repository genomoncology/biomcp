---
base: 053e3aeb367b6a77f0326be52eb46dc13561da73
head: fd9b0f75443eb8b2fc56993d2a7b3f4a7e10c6ae
---
Ticket 256 shipped the article fulltext fallback ladder (PMC HTML + opt-in PDF) but the durable architecture corpus does not define resolver priority, accepted source formats, license/PDF/HTML policy, saved-artifact semantics, or failure visibility. Without this contract, future work on article full-text cannot reason about where new fallbacks belong (entity vs source vs renderer) or how errors should surface to users.

Imported from March ticket 274. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/274-write-article-fulltext-architecture-contract-doc
