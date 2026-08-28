---
base: 05ab27f614c11fd4a194854328c9fef4c021a1a4
head: f40b0e80c660c8406cb15ab7f0110789c7190fc3
---

Added an offline round-trip contract for commands printed by all 12 detail-card families. The contract preserves shell quoting, uses the production Clap parser, and proves malformed printed commands are rejected.
