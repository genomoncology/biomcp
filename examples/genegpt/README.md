# GeneGPT Reproduction

This demo reproduces a GeneGPT-style genomics QA workflow using BioMCP + Pi.

## Scope
- Alias to canonical symbol resolution
- Gene location lookup
- Variant mapping (`rs113488022` -> `BRAF V600E`)
- Gene-disease evidence retrieval
- Therapy context extraction

## Run
```bash
./run.sh
```

## Outputs
- `output.md`: model response
- `stderr.log`: command/tool stderr
- `metrics.json`: elapsed time, inferred tool calls, output word count

## Scoring
```bash
./score.sh output.md
```

The score checks for expected GeneGPT-style anchor facts.
