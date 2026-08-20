---
flow: build
priority: 14
---
# Let the CLI show the MCP tool catalog

There is no way to see the tool catalog an agent receives without writing a JSON-RPC client. Two separate people hit this on 2026-08-19: the exploratory run had to hand-write a client to compare the two builds, and the follow-up verification had to write a second one. Both were rewriting `scripts/measure-mcp-tools.py`, which cannot easily be pointed at an arbitrary installed binary because it depends on the committed tokenizer cache path and on running inside a repository checkout.

The catalog is the real interface for every agent that connects. It carries the tool names, the typed schemas, the enums, the descriptions, and the read-only annotations, and it is what the budget gate measures. Something that important should be inspectable from the binary itself, by a user who has installed it and has no checkout.

This is also the cheapest available fix to a testing gap. Today the gates check BioMCP against fixtures inside the repository. A subcommand that prints the catalog would let the budget, the annotations, and the schema shape be checked against a real installed binary from outside, which is where a user's experience actually lives.

## Done when

- A user with only the installed binary can print the MCP tool catalog, including each tool's name, description, input schema, and annotations.
- The output is machine-readable, so it can be diffed between builds and asserted against in a check that does not need a repository checkout.
- The byte and token measurements the budget gate uses can be reproduced from that output, without the committed tokenizer cache or the repository working directory.
- Printing the catalog does not require starting a server or speaking JSON-RPC.

## Note on scope

Whether `scripts/measure-mcp-tools.py` is then reduced to a thin wrapper, or removed in favour of the subcommand, is a design decision. Say which was chosen and why.
