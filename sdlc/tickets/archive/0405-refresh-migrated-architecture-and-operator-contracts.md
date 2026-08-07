---
flow: build
priority: 5
---
# Refresh migrated architecture and operator contracts

The architecture review found that repo truth still contains migrated-target or stale gate language, while several shipped surfaces lack current contracts: public Rust crate exports, Python surface tests under `spec/surface/`, cache configuration precedence, logging/observability, dependency docs for fulltext conversion, and next-command ownership. These should be repaired in repo docs and pinned by docs parity/ratchet checks where possible.

Completed under March on 2026-06-10, as March ticket 405. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/405-refresh-migrated-architecture-and-operator-contracts
