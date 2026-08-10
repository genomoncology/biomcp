---
flow: build
priority: 8
---
# Render list output from a typed command catalog

`biomcp list --json` is currently reconstructed by scraping BioMCP's own
rendered Markdown. The resulting catalog advertises prose fragments as command
patterns, says `study` is gettable when `get study` does not exist, leaves the
skill command array empty, and duplicates variant helper commands. Discovery
metadata must not be a lossy second parser for documentation.

## Catalog contract

Define one typed catalog for entities, command families, literal examples, and
parameterized templates. Each entry distinguishes:

- a literal executable command that must parse through the production Clap
  tree;
- a template with typed placeholders that is not itself executable; and
- explanatory prose, which is never placed in a command array.

The typed catalog is the source for root/entity `list` JSON and Markdown,
gettable/searchable classification, skill command lists, and the public command
inventory checked by documentation tests. Public projections include README
and docs landing entity tables, the user-guide CLI reference, study
subcommands, and whether each `get` entity actually supports named sections.
`list --json` serializes the model directly; it never parses rendered Markdown.

Remove the false `get study` claim, include the shipped `author` entity and
`study top-mutated`, stop claiming every `get` supports sections, reconcile the
contradictory skill-catalog wording, fill the skill commands, remove duplicates,
and represent filters such as `--region` only as part of a complete command
template. MCP runtime tool inventory and schemas are separate contracts owned
by tickets 0932 and 0933.

## Done when

- Every emitted literal command parses with the production CLI.
- Every placeholder has a type and appears only in a template field.
- JSON and Markdown are projections of the same entries and preserve stable
  ordering.
- Root and per-entity classifications agree with the actual Clap tree.
- README/docs entity tables contain every shipped search/get entity, and every
  public study subcommand and section-capability claim matches the catalog.
- A negative contract fails when a catalog command is removed or made invalid.

## Authorized test changes

Design commits may restate catalog and rendering expectations in
`src/cli/list/tests/*.rs`, `src/cli/tests/next_commands_validity.rs`,
`tests/test_cli_surface_contract_ratchet.py`,
`tests/test_author_surface_docs.py`, public landing/CLI documentation tests,
and documentation consistency tests. They may replace static prose in
`src/cli/list_reference.md` only where
the typed catalog becomes authoritative. Existing useful narrative guidance
may remain outside command arrays.

The src line ceiling may rise by at most 280 lines.
