---
id: '016'
team: biomcp
name: Pathway section truthfulness and guidance
status: ready
priority: 10
flow: build
repo: biomcp
dependencies: []
model-map: default
---

## Why

Post-expansion review found that KEGG pathway section requests can exit
successfully with blank or near-blank human output. `events` remains
Reactome-only in docs, but runtime validation, renderer behavior, and
suggested next commands still allow KEGG flows that feel broken. This breaks
the truthful-degradation contract and undermines user trust in the expanded
pathway surface.

## Scope

### In scope

- Make pathway section availability source-aware across runtime validation,
  render-time messaging, and next-command generation.
- Ensure human and JSON surfaces distinguish unsupported, empty, and
  temporarily unavailable states instead of blank success.
- Align `--help`, `list pathway`, docs, and tests with the chosen source-aware
  pathway contract.

### Out of scope

- Search ranking policy across Reactome + KEGG.
- Default pathway card weight when no section is requested.
- Net-new pathway features or data sources.

## Success Checklist

- [ ] `biomcp get pathway <kegg-id> events` explains why the section is not
      available for KEGG instead of rendering a blank-success page.
- [ ] `biomcp get pathway <id> enrichment` never hides the requested section
      behind an empty human page without explanation.
- [ ] Human suggestions and JSON `_meta.next_commands` are source-aware and do
      not recommend unsupported or already-current flows.
- [ ] CLI help/list/docs describe pathway section availability consistently.
- [ ] Spec/render coverage includes requested-section empty or unsupported
      states.

## Notes

- Review artifacts: `.march/outside-in.md`, `.march/code-review.md`,
  `.march/architecture-review.md`
- Likely touchpoints: `src/entities/pathway.rs`, `src/render/markdown.rs`,
  `templates/pathway.md.j2`, `src/cli/{mod.rs,list.rs}`,
  `docs/user-guide/pathway.md`
