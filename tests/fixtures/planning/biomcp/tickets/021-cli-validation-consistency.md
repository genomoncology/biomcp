---
id: '021'
team: biomcp
name: Normalize CLI usage errors and remediation
status: ready
priority: 6
flow: build
repo: biomcp
dependencies: []
model-map: default
---

## Why

The post-expansion review found usage validation drift at the shell boundary:
`biomcp search pathway --help` presents `[QUERY]` as optional even though the
runtime requires it, the missing-query remediation example is malformed for
multi-word queries, and invalid-usage exits are not categorized consistently
between clap errors and runtime section errors. This makes scripting and user
recovery harder than it should be.

## Scope

### In scope

- Make the required/optional query contract consistent between clap help and
  runtime validation for pathway search.
- Fix remediation examples so multi-word queries are quoted correctly.
- Decide and apply a consistent invalid-usage exit-code/error-style policy for
  missing query and invalid section cases.
- Update docs and tests to match the chosen CLI validation contract.

### Out of scope

- Search relevance changes.
- Broader UX copy work unrelated to invalid usage and recovery guidance.

## Success Checklist

- [ ] `biomcp search pathway --help` matches runtime behavior for missing
      queries.
- [ ] Recovery guidance quotes multi-word example queries correctly.
- [ ] Missing-query and invalid-section paths follow a deliberate exit-code and
      error-style policy.
- [ ] CLI docs/tests cover the chosen validation behavior.

## Notes

- Review artifacts: `.march/outside-in.md`, `.march/architecture-review.md`
- Likely touchpoints: `src/cli/mod.rs`, `src/entities/pathway.rs`,
  `docs/user-guide/pathway.md`, `docs/reference/error-codes.md`
