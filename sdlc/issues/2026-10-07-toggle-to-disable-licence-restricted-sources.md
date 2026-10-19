# Feature request: toggle to disable licence-restricted sources such as KEGG

External issue: genomoncology/biomcp#290
Reporter: mjafin, 2026-10-07
Status: resolved.

Opened as: open (triaged 2026-10-07; no public reply posted — postings need Ian's approval)

## Report

The reporter asks for the best mechanism to keep BioMCP from using KEGG
or other sources that require a commercial licence in agentic
workflows, suggesting a switch of some sort.

## Triage

The request is in scope for 0.9.x behavior: source selection is a
CLI/MCP surface question, not new biology. The repo already carries the
building blocks — sources are registry-driven, and the WHO degrade path
(1304) plus the citation fallback (1302) show how a source can drop out
honestly while the answer says which source went silent.

Root question for design: what does a caller mean by "disable"? Three
readable shapes:

1. A config or environment switch (for example
   `BIOMCP_DISABLED_SOURCES=kegg,...`) that removes the source from
   discovery and marks its sections unavailable with the reason
   "disabled by caller", so output stays honest instead of silently
   shrinking.
2. A licence-acknowledgement mode where restricted sources stay off
   until the caller opts in.
3. Per-call filtering, which fits MCP tool schemas less well.

Shape 1 is the smallest honest start and matches the existing degrade
conventions. The source registry should be the single place that reads
the switch, so every caller (CLI, MCP, JSON) sees the same omission and
the same reason.

## Success criteria (for the eventual ticket)

- A caller can name restricted sources to exclude, and those sources
  never answer.
- Every response that would have used a disabled source names the
  source and the reason, following the 1302/1304 degrade wording.
- Discovery and section outcomes honor the switch uniformly across CLI,
  MCP, and JSON.
- The default changes nothing for callers who set no switch.

## Disposition

Hold for the 0.9.2 blocker queue (2016-2021) to clear, then draw a
ticket number from `pm ticket new` with these criteria. The public
reply on #290 needs Ian's approval before posting; the 0.9.2 outcome
promises every external issue an answer.

## Release dispositions (2026-10-19)

- Source-disable request: FIXED by 2061 at 35f42b858 and shipped in 0.9.3. The source registry applies configured exclusions across CLI and MCP and reports the disabled reason.
- External issue #290: CLOSED after the approved public reply. The open-era status and approval wording above are historical.
