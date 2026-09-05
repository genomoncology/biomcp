---
flow: build
priority: 3
deps: [1147]
---

# Correct reversed search commands with the canonical form

`biomcp article search` suggests `article batch`, and `biomcp trial search` suggests `article`. Neither suggestion fixes the command. BioMCP already knows that article and trial are searchable entities, so it can direct the caller to `biomcp search article` or `biomcp search trial`. The current behavior and parser cause appear in `sdlc/issues/reversed-search-grammar-suggests-an-unrelated-command.md` at commit `fe2f9fc1`.

## Required behavior

When a caller enters `<searchable-entity> search`, BioMCP reports the canonical `biomcp search <entity>` form. BioMCP preserves the remaining arguments when the reordered canonical command parses. The correction replaces unrelated similarity suggestions in human-readable and JSON errors.

The canonical grammar remains `biomcp search <entity>`. BioMCP does not accept every reversed command as an alias.

Done, observably:

- `biomcp article search` points to `biomcp search article`.
- `biomcp trial search` points to `biomcp search trial`.
- Every searchable entity receives the same correction for the same reversal.
- The suggested command parses when the original command supplied valid search arguments.
- Human-readable and JSON errors carry the same actionable correction.
- Other parse errors retain their existing diagnostics unless the same safe correction applies.

Boundary: this ticket improves recovery from one common word-order mistake. It does not add reversed aliases, rename the established `search|get|batch <entity>` grammar, replace the parser, or redesign unrelated command errors.
