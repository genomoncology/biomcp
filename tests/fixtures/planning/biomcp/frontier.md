# BioMCP Frontier

Bounded fronts the team should maintain queue depth against.
Each front is linked to a goal and points into repo analysis for substance.
This file does not list individual tickets or date commitments.

---

## G002 — Give Back to the Community

> Real people using BioMCP for real biomedical work.

### Front: Skills Reliability

Skills are the highest-leverage community surface. A researcher who runs one
skill invocation and gets a useful investigation workflow is the primary G002
success signal. Skills should produce correct, well-formatted output and stay
current with the source-aware CLI surface they reference.

Source: `design/functional/overview.md` (Skills section),
`design/ux/cli-reference.md` (demo workflows)

Harvest: tickets that fix skill output correctness, update skill docs when the
CLI surface changes, or add new investigation patterns that serve real use cases.

### Front: Newcomer Discoverability

MCP ecosystem directories, biomcp.org content, install ergonomics, terminal
readability, and help/next-step accuracy determine whether new users arrive and
stay. A newcomer should be able to install BioMCP, run a first command, and
understand what to do next in under five minutes.

Source: `design/functional/overview.md` (Audience and Done-Enough sections),
`design/ux/cli-reference.md` (quick-start pattern)

Harvest: tickets improving install UX, `biomcp health` messaging, help/error
accuracy, output readability, and ecosystem listing presence.

---

## G003 — Ship v1.0

> Skills, CLI, docs, and tests meet a standard you'd hand to a colleague.

### Front: Source Truthfulness and Stability

Source-expansion regressions on shipped surfaces need user-trust fixes before
v1.0. The emphasis is truthful section behavior, concise default cards,
context-aware next steps, and search behavior that does not bury exact source
matches behind generic merge order.

Source: `design/technical/source-integration.md` (Section-First Entity
Integration, Graceful Degradation and Timeouts),
`design/ux/cli-reference.md` (operator flows)

Harvest: bug-fix tickets that correct blank-success states, misleading section
guidance, heavy default cards, or user-visible ranking regressions without
adding net-new surface area.

### Front: Operator Proof Surfaces

Every shipped source expansion must land with intentional proof at the operator
boundary: spec coverage for the user-visible flow, targeted tests for empty or
unsupported states, `biomcp health` when operators rely on readiness, and
`contract-smoke` probes when live checks are stable enough.

Source: `design/technical/source-integration.md` (Source Addition Checklist),
`design/technical/overview.md` (Verification section)

Harvest: tickets extending health, contract-smoke, targeted tests, and
release-facing docs so operator promises match implementation.

### Front: Documentation Completeness

Architecture, CLI docs, and reference docs must share one entity-to-source
story before v1.0. User and operator docs should not promise source coverage,
progressive disclosure, or health guarantees that the live CLI does not honor.

Source: `design/functional/overview.md`, `design/technical/overview.md`,
`docs/reference/data-sources.md`

Harvest: docs tickets that align entity/source inventory, proof-surface
promises, and source-specific section guidance. Prioritize user-visible gaps
that block trust.

### Front: Paper and Citation

G003 success criteria include a published paper or citation. This is a separate
work stream from code quality and can proceed in parallel once the CLI is stable.

Source: `design/functional/overview.md` (Audience section — who the paper
should address)

Harvest: write-up, submission, and promotion tickets when the CLI is v1.0-ready.

---

## Harvest Guidance

When triaging inbox items against this frontier:
- If the item improves skill output or newcomer experience → G002 fronts
- If the item fixes source truth, operator proof, or docs drift toward v1.0 →
  the first three G003 fronts
- If the item adds a net-new feature not tied to G002/G003 success criteria →
  hold for a future goal or strategic decision
- G001 (awareness/marketing) is currently deferred; items that fit it should
  be noted in triage but not queued until G001 becomes active
