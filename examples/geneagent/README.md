# GeneAgent Reproduction

This demo reproduces a GeneAgent-style chain from variant findings to pathway and drug context.

## Scope
- Variant significance summary
- Pathway grounding (e.g., MAPK/RAS)
- Drug/therapy context from target/pathway signal
- Protein-level context mention

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

The score checks for expected pathway + therapy anchors.
