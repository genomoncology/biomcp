---
flow: build
priority: 5
---

# The MCP tool schema does not name the sections it accepts, and agents invent them

Agents driving BioMCP over MCP spend a large share of their calls guessing section names that do not exist.

Measured on 2026-09-01. BioMCP `0.8.25` served four MCP tools over stdio via `biomcp serve`. Claude Code `2.1.252` in headless mode drove them across 31 biomedical tasks, one fresh process per task, model `claude-sonnet-5`, with no other tools available. Of 179 BioMCP tool calls, **52 failed, 29.1%**. The largest single cause was an invented section name, at **19 of the 52**: `guidelines`, `guidance`, `conditions` and `ontology` were all requested and none exist.

The valid tokens are known to the program. `get trial <id> --help` and the error message itself both enumerate them:

```
Error: Invalid argument: Unknown section "..." for trial.
Available: eligibility, contacts, locations, outcomes, arms, references, all
```

So the agent learns the answer only by failing first. An agent that never reads a `--help` and never sees an error has no way to know the set, because the tool schema it does receive does not carry it.

This has a measured downstream cost beyond wasted calls. In one task the agent inverted a verb and noun, five of its nine calls errored, it never ran a working trial search, fell back to a disease card and reported **1** recruiting retinoblastoma trial. The correct number is 22.

## Required behavior

An agent reading the tool definitions, and nothing else, can tell which section tokens an entity accepts before it calls anything.

The set an agent is told about and the set the program accepts are the same set, and they cannot drift apart unnoticed.

## Done, observably

- The MCP tool definitions name the accepted section tokens for each entity that takes sections.
- A section token added or removed in the program changes what the schema advertises, without anyone editing two places by hand.
- The suite fails if the advertised set and the accepted set disagree.

## Boundary

This ticket does not change which sections exist, does not rename any token, and does not change error text beyond what consistency requires. The catalog has an enforced size ceiling in `scripts/measure-mcp-tools.py`; this work fits under the existing ceiling rather than raising it, and if it cannot, say so rather than widening the gate.
