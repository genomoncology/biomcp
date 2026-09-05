---
flow: build
priority: 5
---

# A rejected prose keyword has no way through

## Goal

A caller whose keyword legitimately contains `gene:`, `disease:`, or `drug:` as ordinary prose can act on the rejection. Today the message tells them to do something that does not express what they asked for.

Observed at commit `f68d8832` with the release binary `0.9.0-dev.6+gf68d8832` on 2026-09-05:

```text
$ biomcp search article -k "Alzheimer disease: mechanisms"
Error: Invalid argument: keyword is provider-neutral and does not accept disease: filter syntax. Use --disease melanoma for CLI or raw MCP, or the typed MCP field, for example "disease":"melanoma".
$ echo $?
2
```

`review of drug: safety` and `gene: expression profiling` are rejected the same way. A title-shaped phrase is a normal literature query, and `--disease melanoma` does not search for it.

This is inside the boundary record 1100 accepted on purpose: a reserved label is recognized when it starts the trimmed value or follows whitespace or `(`. The recognizer is earning its keep — the study behind that record counted fifteen of fifty-two failed calls putting field syntax in a plain-text field. The whole compatibility corpus in record 1100 still behaves as written, verified at this commit: `NM_004333.6:c.1799T>A`, `protein:protein interaction`, `oncogene:RB1`, `MYGENE:RB1`, `ratio 1:2`, and `BRAF[variant]` all reach the provider, and `gene:RB1`, `GENE:RB1`, `melanoma (gene:RB1)`, and `gene:"RB1"` are all rejected with exit 2.

Record 1100 also documents an escape: literal quote bytes before the label stop the match. It works today.

```text
$ biomcp search article -k '"disease: mechanisms"'
# Articles: keyword="disease: mechanisms", exclude_retracted=true, sort=relevance, ...
```

Nothing in the rejection mentions it, so a caller cannot find it from the error.

## The choice to settle

Two ways to close this.

1. Teach the escape in the diagnostic, so the rejection tells the caller how to send the value as literal text as well as how to use the typed filter. Cost: the message gets longer, and quoting rules are a thing an agent can still get wrong.
2. Narrow the recognizer so a reserved word followed by a colon is only rejected when what follows looks like a filter value. Cost: it re-opens a lexical boundary that passed design review and code review, and every narrowing lets some share of the original fifteen failures back through.

Take option 1. The escape already exists and already works, so this is a wording change against behavior that is proven, and it leaves the reviewed recognizer alone.

## Done, observably

- The rejection for a reserved keyword label tells the caller both how to use the structured filter and how to send the value as literal keyword text.
- The diagnostic gives copyable, surface-correct examples for the CLI/raw MCP string and for a typed MCP JSON request. It does not interpolate or echo the rejected value.
- Each literal-text instruction, followed exactly, reaches a request-capturing provider fixture rather than producing a second rejection. The captured request preserves the intended quote-bounded literal phrase and contains no structured gene, disease, or drug filter. The regression covers CLI, raw MCP, and typed MCP request encoding instead of stopping at the validator.
- Every value in record 1100's compatibility corpus keeps reaching the provider, and every value in its rejected corpus keeps exiting 2.
- Existing exact diagnostic assertions for article CLI, search-all, JSON error envelopes, and MCP are updated together.
- Nothing about the gene-value rule or the author, affiliation, and journal recognizers changes.

## Boundary

This ticket changes what the reserved-label rejection says. It does not change the recognizer's lexical boundary, add a quote grammar, strip quote bytes before sending them to a provider, or touch which values are rejected.
