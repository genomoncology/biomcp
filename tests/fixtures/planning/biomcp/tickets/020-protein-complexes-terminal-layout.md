---
id: '020'
team: biomcp
name: Improve protein complexes terminal layout
status: ready
priority: 6
flow: build
repo: biomcp
dependencies: []
model-map: default
---

## Why

`biomcp get protein <accession> complexes` now returns valuable ComplexPortal
data, but the human-readable layout is too wide and dense for comfortable
terminal use. Long complex names and component lists create wrapping noise,
and the related-command guidance can repeat the command the user just ran.

## Scope

### In scope

- Redesign the human-readable complexes output for long names and component
  lists so the important signal is still scannable in a terminal.
- Make related/next-command guidance context-aware for the complexes view.
- Add render/spec coverage and doc examples for the improved layout.

### Out of scope

- New ComplexPortal fields or filtering features.
- JSON schema changes unless layout work makes a metadata fix unavoidable.

## Success Checklist

- [ ] Protein complexes output remains useful in an 80-120 column terminal.
- [ ] Related/next-command guidance does not repeat the current complexes
      command.
- [ ] Render/spec coverage locks in the intended human layout.
- [ ] Docs/examples reflect the improved complexes presentation.

## Notes

- Review artifacts: `.march/outside-in.md`, `.march/code-review.md`
- Likely touchpoints: `src/render/markdown.rs`, `templates/protein.md.j2`,
  `docs/user-guide/protein.md`, `spec/16-protein.md`
