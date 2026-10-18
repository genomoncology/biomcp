---
priority: 3
---

# 2210 — Try toml 1.x

Status: OPEN.

Milestone: 0.9.4

## Outcome

Moving BioMCP to toml 1.x either lands with measured build and dependency numbers no worse than today's, or is recorded as not worth it.

- Evidence: the 0.9.3 dependency design listed a toml 1.x trial; 2072 left it out of 0.9.3 unless a measured ticket was filed.
- Changes: `Cargo.toml`, `Cargo.lock`, the configuration readers that parse TOML, and `CHANGELOG.md` if it lands.
- Proof: configuration and cache settings parse identically (existing cases plus one for each setting file), and compile time, crate count and duplicate-version count before and after on one build machine.
