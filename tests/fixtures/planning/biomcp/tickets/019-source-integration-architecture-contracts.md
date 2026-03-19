---
id: '019'
team: biomcp
name: Refresh source-aware integration architecture contracts
status: ready
priority: 7
flow: build
repo: biomcp
dependencies: []
model-map: default
---

## Why

The architecture corpus now disagrees with itself about the entity-to-source
map, the strength of `biomcp health`, how universal the shared HTTP-client
pattern is, and what proof surfaces are required for source additions. There
is also no durable contract for source-aware sections or multi-source ranking,
so behavior has become implementation-defined instead of planned.

## Scope

### In scope

- Refresh the architecture/system-shape docs so they agree on the current
  source inventory after KEGG, ComplexPortal, HPA, and gnomAD expansion.
- Define where source-aware section availability and unsupported/empty state
  rules live, and how help/list/render/docs derive from that contract.
- Define the pathway multi-source ranking contract at the architecture level.
- Clarify proof-surface and non-JSON transport guidance for future source work.

### Out of scope

- Implementing the code fixes themselves.
- Broad documentation refreshes unrelated to source integration.

## Success Checklist

- [ ] `design/functional/overview.md`, `design/technical/overview.md`, and
      `design/technical/source-integration.md` agree on the shipped source map.
- [ ] The architecture docs define a durable source-aware section contract.
- [ ] The architecture docs define the intended proof surface for source
      additions and the meaning of `biomcp health`.
- [ ] The architecture docs cover non-JSON transport/parsing guidance added by
      recent source work.

## Notes

- Review artifact: `.march/architecture-review.md`
- Likely touchpoints: `design/functional/overview.md`,
  `design/technical/overview.md`, `design/technical/source-integration.md`,
  `docs/concepts/progressive-disclosure.md`
