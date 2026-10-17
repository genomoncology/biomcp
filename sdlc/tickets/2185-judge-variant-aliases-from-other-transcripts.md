---
priority: 3
---

# 2185 — Judge variant aliases from other transcripts

Status: OPEN.

Milestone: 0.9.4

## Outcome

Variant article search uses an alias from another transcript only when a clinical review rules that it names the same variant.

- Evidence: research's spot check: `BRCA1 F1761I` expands to aliases such as `BRCA1 F71I` and `BRCA1 c.211T>A` from other transcripts, which pull papers matched only by those names, for example 34829431; identity checks mark them unverified.
- Starts from: a clinical reviewer's ruling on which transcript aliases name the same variant. The PM routes that question; the team does not decide it.
- Changes: claims the alias expansion in `src/entities/article/variant_search.rs` and its tests, `CHANGELOG.md`, after the ruling.
- Proof: recorded replies show the ruled behavior for BRCA1 F1761I.
