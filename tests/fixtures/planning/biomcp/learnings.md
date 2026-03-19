# BioMCP Learnings

## 2026-03-19 — Post-expansion holistic review

- Source-specific section availability needs one source of truth that drives
  clap help, `list` output, runtime validation, render guidance, and JSON
  metadata. Generic global section helpers drift once one entity spans
  multiple upstreams.
- Progressive disclosure must be enforced in transforms/renderers, not only in
  docs. If a default card already inlines a deep section, user docs and
  suggested next steps will drift quickly.
- Multi-source search ranking is part of the product contract. Exact-match
  behavior and source ordering should be explicit rather than an accident of
  merge order or per-source budgeting.
- A source addition is not done until proof surfaces are updated
  intentionally: specs for the user-visible flow, targeted tests for empty or
  unsupported states, `biomcp health` when operators rely on readiness, and
  `contract-smoke` when the upstream is stable enough for live probes.
- Next-command helpers need semantic review, not only syntax checks. A command
  that repeats the current view or suggests an unsupported section still burns
  user trust even if it parses.
- Terminal-readable output needs separate review from parseable output. Long
  biomedical names/components can pass structural tests while remaining hard
  for humans to scan.
