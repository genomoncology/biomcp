# How To: Predict Effects

BioMCP can request functional-impact predictions through the variant `predict` section.

## Requirements

- Set `ALPHAGENOME_API_KEY`.
- Prefer a resolvable genomic variant identifier.

## Example

```bash
export ALPHAGENOME_API_KEY="..."
biomcp get variant "chr7:g.140453136A>T" predict
```

## Alternate input

```bash
biomcp get variant "BRAF V600E" predict
```

## Validation behavior

BioMCP validates identifiers and dates before expensive network work.
If a variant cannot be resolved to a supported prediction path, an explicit error or warning is returned.

## Practical guidance

- Use prediction output as one signal, not a final interpretation.
- Pair predictions with variant annotations and literature context.
