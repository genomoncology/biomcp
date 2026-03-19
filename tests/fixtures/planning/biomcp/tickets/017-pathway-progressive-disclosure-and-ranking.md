---
id: '017'
team: biomcp
name: Restore pathway progressive disclosure and ranking
status: ready
priority: 9
flow: build
repo: biomcp
dependencies: []
model-map: default
---

## Why

The KEGG default pathway card currently dumps a large gene wall even when the
user did not ask for `genes`, and pathway search can rank weaker Reactome
matches ahead of exact KEGG hits such as `"Pathways in cancer"`. The result is
a pathway experience that feels noisy, heavy, and less trustworthy than the
docs promise.

## Scope

### In scope

- Keep default pathway cards concise so deep sections such as `genes` stay
  behind explicit section requests unless there is a documented exception.
- Define and implement a cross-source pathway search merge policy that favors
  exact matches and avoids hard-coded Reactome-first relevance drift.
- Update pathway docs/specs to reflect the chosen default-card and ranking
  behavior.

### Out of scope

- New pathway sources or enrichment features.
- Generic `search all` ranking changes outside pathway search.

## Success Checklist

- [ ] Default KEGG pathway cards no longer inline the full gene section unless
      the user explicitly requests it.
- [ ] Exact KEGG title matches are not buried behind weaker Reactome hits for
      common pathway queries.
- [ ] Pathway docs and list/help flows match the shipped
      progressive-disclosure behavior.
- [ ] Spec coverage exercises default pathway cards plus KEGG search ranking
      expectations.

## Notes

- Review artifacts: `.march/outside-in.md`, `.march/code-review.md`,
  `.march/architecture-review.md`
- Likely touchpoints: `src/transform/pathway.rs`, `src/entities/pathway.rs`,
  `spec/14-pathway.md`, `docs/user-guide/pathway.md`
