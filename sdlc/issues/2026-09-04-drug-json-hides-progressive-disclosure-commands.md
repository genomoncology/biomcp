# Drug JSON hides the commands that reveal current approval and label data

`biomcp --json get drug eflornithine` showed a 1990 approval date and two older indications on `biomcp 0.9.0-dev.6` on 2026-09-04. Its `_meta.next_commands` listed literature, trials, adverse events, pharmacogenomics, and ODC1. It did not list any drug section command.

Two omitted section commands contained the current regulatory facts:

```bash
biomcp --json get drug eflornithine approvals
biomcp --json get drug eflornithine label
```

The approvals section returned `NDA215500` with the original submission approved on 2023-12-13. The label section returned Iwilfin's high-risk neuroblastoma indication. The human-readable card prints a `More:` section, but `src/cli/drug/render.rs::render_loaded_card` passes only `related_drug` to the JSON serializer. The JSON path never adds `sections_drug` commands.

## Recommended design

Build drug next commands from the requested sections and related pivots through one shared function. Use that function for Markdown, JSON, MCP, and batch responses. Include the bounded visible section commands and the existing related commands. Keep `get drug <name> all` discoverable when the response omits sections.

The cost is a longer `_meta.next_commands` list. A bounded ordered list costs less than hiding the source that corrects an incomplete overview.

## Done, observably

- Default drug JSON suggests `label`, `regulatory`, or `approvals` according to the same ordering used by the human-readable card.
- Requesting one section removes that exact duplicate and leaves useful remaining sections.
- Markdown, JSON, MCP, and batch paths share one tested next-command builder.
- Every emitted command passes the parser property test.
