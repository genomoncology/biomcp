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
current with the CLI surface they reference.

Source: `design/functional/overview.md` (Skills section),
`design/ux/cli-reference.md` (demo workflows)

Harvest: tickets that fix skill output correctness, update skill docs when the
CLI surface changes, or add new investigation patterns that serve real use cases.

### Front: Newcomer Discoverability

MCP ecosystem directories, biomcp.org content, install ergonomics, and README
quality determine whether new users arrive. A newcomer should be able to install
BioMCP, run a first command, and understand what to do next in under five minutes.

Source: `design/functional/overview.md` (Audience and Done-Enough sections),
`design/ux/cli-reference.md` (quick-start pattern)

Harvest: tickets improving install UX, `biomcp health` messaging, help text
accuracy, and ecosystem listing presence.

---

## G003 — Ship v1.0

> Skills, CLI, docs, and tests meet a standard you'd hand to a colleague.

### Front: Correctness and Stability

Bug-free on the core entity workflows (variant lookup, trial search, drug
safety, article search, cross-entity pivots) before cutting v1.0. CI green,
spec suite passing, no known regressions in the main workflows.

Source: `design/technical/overview.md` (Verification section)

Harvest: targeted bug-fix tickets identified by spec failures, contract-smoke
failures, or user-reported regressions. Do not add features here — fix what
exists.

### Front: Documentation Completeness

`search all`, cross-entity pivots, and all 14 skills must be fully documented
before v1.0. This includes both the repo-local `docs/` tree and biomcp.org.

Source: `design/functional/overview.md` (Command Grammar and Skills sections),
`design/ux/cli-reference.md`

Harvest: docs tickets that close gaps between the CLI surface and published
documentation. Prioritize user-visible gaps that block newcomers.

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
- If the item fixes a bug or closes a doc gap toward v1.0 → G003 fronts
- If the item adds a net-new feature not tied to G002/G003 success criteria →
  hold for a future goal or strategic decision
- G001 (awareness/marketing) is currently deferred; items that fit it should
  be noted in triage but not queued until G001 becomes active
