---
flow: build
priority: 8
---
# Return successful items from partially failed batches

The top-level `biomcp batch` uses fail-fast joining for every entity. A request
such as `batch gene BRAF,ZZZZZZNOTGENE` discards the valid BRAF result when the
other identifier fails. Independent batch work should not erase completed
biomedical results.

## Batch contract

Run all validated item requests concurrently and preserve input order. JSON is
one stable envelope:

```json
{
  "summary": {"total": 2, "succeeded": 1, "failed": 1},
  "items": [
    {"input": "BRAF", "status": "ok", "result": {}},
    {"input": "ZZZZZZNOTGENE", "status": "error", "error": {}}
  ]
}
```

Each success keeps its entity result and `_meta`; each failure uses the normal
safe structured error fields. Human output renders every success and every
failure in order, followed by the same summary.

All success exits zero. Any item failure exits one after writing the complete
result. Whole-command validation errors such as an unknown entity, an invalid
batch size, or invalid shared sections still fail before item requests begin.
The separate `biomcp article batch` surface adopts this exact envelope and exit
contract through the same helper. It must not retain its independent
`try_join_all` fail-fast path.

## Done when

All-success, mixed, and all-failure local cases pass for each supported
top-level batch entity and for `biomcp article batch`. A slow first item and
fast later item prove concurrency
does not reorder output. One failure never cancels or hides another completed
item. JSON and human exit behavior agree and no routine test uses public
network.

## Authorized test changes

Design commits may replace fail-fast and JSON-array expectations in
`src/cli/system/dispatch.rs`, `src/cli/shared.rs`,
`src/cli/tests/outcome.rs`,
`src/cli/tests/next_commands_json_property/*.rs`,
`src/cli/response_contract.rs`, `src/entities/article/batch.rs`,
`src/cli/article/dispatch.rs`, and system/article CLI tests. Existing
per-entity rendering and batch-size validation remain covered.

The src line ceiling may rise by at most 300 lines.
