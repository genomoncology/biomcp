---
flow: build
priority: 3
---

# Give article batching one canonical command and contract

BioMCP exposes two different article batch commands. `biomcp article batch <id>...` returns compact shortlist cards and accepts space-separated identifiers. `biomcp batch article <id,...>` performs ordinary article gets and accepts one comma-separated argument. The commands also advertise different limits and have separate documentation. The CLI reference defines `batch <entity>` as the generic bulk-operation grammar, but article workflows and generated suggestions teach the other form. A caller can therefore choose between two plausible commands and receive different behavior without asking for that difference.

## Required behavior

`biomcp batch article` becomes the canonical article batch surface. It offers the existing compact shortlist behavior and the existing detailed get behavior as explicit modes under one command. The selected mode determines the response contract and limit. Word order does not select hidden semantics.

`biomcp article batch` remains a compatibility route for existing scripts and produces the same result as the matching canonical mode. Help, command references, examples, and generated next commands teach only the canonical form. The compatibility route tells interactive callers which form to use for new work without corrupting machine-readable output.

Done, observably:

- One documented `batch article` surface can request compact shortlist cards or detailed article results.
- The requested mode has one result shape, limit policy, per-item error policy, and exit-status policy regardless of the accepted route.
- Existing valid `article batch` calls continue to run and preserve their requested order.
- New BioMCP output does not generate `article batch` as a next command.
- Help and durable command references no longer present two article batch commands as unrelated choices.
- Other `batch <entity>` commands retain their current behavior.

Boundary: this ticket consolidates the public article batch contract without removing the compatibility route. It does not rename relationship pivots such as `article citations`, change ordinary article detail, redesign batching for every entity, or require a breaking response change for existing scripts.
