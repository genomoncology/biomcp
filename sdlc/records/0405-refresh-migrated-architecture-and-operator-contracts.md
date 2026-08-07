---
base: 57b6c95db66af120c54ff5f7b546f002721b3af0
head: 9fea820498e14ba2231238a42516849346abb470
---
The architecture review found that repo truth still contains migrated-target or stale gate language, while several shipped surfaces lack current contracts: public Rust crate exports, Python surface tests under `spec/surface/`, cache configuration precedence, logging/observability, dependency docs for fulltext conversion, and next-command ownership. These should be repaired in repo docs and pinned by docs parity/ratchet checks where possible.

Imported from March ticket 405. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/405-refresh-migrated-architecture-and-operator-contracts
