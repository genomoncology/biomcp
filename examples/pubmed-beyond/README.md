# PubMed & Beyond Reproduction

This demo reproduces a literature-intelligence workflow inspired by PubMed & Beyond.

## Scope
- Retrieve representative BRAF/melanoma literature
- Extract evidence points for BRAF V600E clinical relevance
- Mention treatment or resistance angle from article context

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

The score checks for expected literature + clinical evidence anchors.
