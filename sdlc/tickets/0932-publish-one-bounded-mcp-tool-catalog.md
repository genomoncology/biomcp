---
flow: build
priority: 9
deps: ["0919", "0930"]
---
# Publish one bounded MCP tool catalog

The server exposes seven tools, while setup documentation, an active blog, the
MCPB contract, and tests still claim exactly one. The current serialized
`tools/list` measured 23,515 bytes and about 6,052 `cl100k_base` tokens; the raw
`biomcp` description alone is 17,637 characters. The public claim of roughly
800 tokens and a 95 percent reduction is not reproducible.

## Catalog contract

Retain the intentional seven-tool surface:

- `biomcp`
- `search`
- `get`
- `variant_normalize_car`
- `variant_erepo`
- `gene_cspec`
- `variant_articles`

Define those names, descriptions, annotations, and schemas in one typed router
catalog. Runtime `tools/list`, server instructions, MCP reference/setup pages,
applicable MCPB/registry metadata, and inventory tests consume that catalog or
a checked generated projection. No prose file or test keeps an independent
tool count.

The raw `biomcp` tool is a compact escape hatch: its description is at most
4,000 UTF-8 bytes and directs callers to bounded `list` discovery instead of
embedding the full Markdown command reference. The complete serialized
`tools/list` is at most 16,000 bytes and 4,000 `cl100k_base` tokens. These are
ratcheted ceilings, not marketing estimates.

## Done when

- A local initialize/`tools/list` measurement proves exactly the catalog tools,
  stable ordering, unique names, byte ceiling, and token ceiling.
- Server instructions describe all seven tools and favor bounded typed tools
  before the raw escape hatch.
- Claude/MCP setup, MCP reference, manifest metadata, and the active blog state
  the same inventory and include a current reproducible context measurement.
- Tests that currently freeze a one-tool story are explicitly restated; a new
  tool without catalog, docs, annotations, and budget fails.
- The catalog adds no provider call and needs no public network.

## Authorized test changes

Design commits may restate the current one-tool assertions in
`tests/test_documentation_consistency_audit_contract.py`,
`tests/test_directory_submission_contract.py`, MCP specs/contracts,
`manifest.json`, and public MCP documentation. They may replace
`src/cli/list_reference.md` as the raw tool description only where the typed
catalog/list route becomes authoritative. Existing protocol and safety
annotations remain covered.

The src line ceiling may rise by at most 280 lines.
