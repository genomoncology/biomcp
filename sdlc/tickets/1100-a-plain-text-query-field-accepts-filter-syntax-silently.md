---
flow: build
priority: 3
---

# A plain-text query field accepts filter syntax and answers as though it were text

An agent that writes `gene:RB1` into a query field gets a search for the literal characters, not a filter, and nothing says so.

Measured on 2026-09-01, in the same 31-task MCP study described in the sections ticket. Of 179 BioMCP tool calls, 52 failed. **15 of the 52 wrote field-scoped filter syntax into a plain-text query field.** A further 6 passed free text where a symbol is required, for example `TPMT mercaptopurine` into a gene field.

The pattern behind both is the same. BioMCP's grammar is compositional and reads like a query language, so an agent assumes it accepts the filter forms every other query language accepts. It does not, and the failure gives the agent nothing to correct against.

Together with the invented-section bucket, these two shapes are 65% of every failed call in the study.

## Required behavior

A query value that is field-scoped filter syntax is recognised as such and answered with what to write instead, rather than searched for as literal characters.

A field that requires a single symbol says so when it is handed something that is plainly not one.

An agent can correct a malformed query from the response alone, without a second exploratory call.

## Done, observably

- A query carrying field-scoped filter syntax produces a response naming the correct form for that filter.
- A field requiring a symbol rejects an obviously non-symbol value with a message that shows a correct value.
- A legitimate query that happens to contain a colon still searches normally.

## Boundary

This ticket does not add a query language and does not make BioMCP accept filter syntax. Deciding to accept the syntax rather than reject it clearly is a design choice, and the design stage owns it; either answer satisfies this ticket as long as the agent can correct itself from one response. Ordinary successful queries keep their current behaviour.
