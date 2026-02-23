# TrialGPT Reproduction

This demo reproduces a TrialGPT-style trial matching workflow with BioMCP trial search/get.

## Scope
- Find active BRAF V600E melanoma trials
- Summarize key eligibility details
- Capture phase/intervention context
- Provide matching rationale for a patient profile

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

The score checks for trial identifiers and matching-related evidence.
