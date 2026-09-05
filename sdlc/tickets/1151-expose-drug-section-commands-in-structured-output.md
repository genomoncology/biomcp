---
flow: build
priority: 6
deps: [1161]
---

# Expose drug section commands in structured output

## Goal

Structured drug responses tell agents how to open regulatory, approval, label, and other available sections. On 2026-09-04, `biomcp --json get drug eflornithine` showed older overview facts and suggested literature, trials, adverse events, pharmacogenomics, and ODC1. It did not suggest the `approvals` or `label` commands that returned the current Iwilfin approval and indication. The reproduction and code evidence came from `sdlc/issues/2026-09-04-drug-json-hides-progressive-disclosure-commands.md` in commit `f8ff2a78`.

## Desired functionality

Drug responses expose a short ordered set of runnable commands for sections that were not loaded. Human-readable, JSON, MCP, and batch responses present consistent section discovery. A requested section does not immediately suggest the identical command again. Related-entity commands remain available within a bounded list.

## Success criteria

- Default structured output for eflornithine suggests commands that open its approval, label, and regulatory information.
- Every suggested command parses and requests the named drug section.
- A response does not repeat the exact section command that produced it.
- Human-readable, JSON, MCP, and batch responses expose the same available section choices.
- Existing related article, trial, adverse-event, pharmacogenomic, and gene pivots remain discoverable.
- The next-command list remains bounded and ordered.

## Boundaries

This ticket changes command discovery. It does not load every drug section by default, change regulatory facts, add a regulatory provider, or alter drug matching.
