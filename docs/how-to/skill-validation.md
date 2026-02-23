# Skill Validation Guide

This guide defines how to evaluate whether a BioMCP skill run is complete and trustworthy. Validation is checklist-driven and attached directly to each skill markdown file.

## Validation Model

Each skill should provide:

1. **Quick Check** to confirm command path health.
2. **Full Workflow** with explicit step intent.
3. **Validation Checklist** with concrete expected outcomes.

A run is considered valid when checklist items can be traced to command output.

## Reviewer Checklist

Use this short rubric when reviewing a skill execution log:

| Criterion | Pass condition |
|-----------|----------------|
| Command fidelity | Steps match the skill workflow commands |
| Evidence traceability | Output includes IDs (PMID/NCT/variant IDs) where relevant |
| Clinical relevance | Summary ties findings back to disease/variant/drug context |
| Constraint awareness | Eligibility/safety/limitations noted when applicable |
| Reproducibility | Another reviewer can rerun commands and get equivalent structure |

## Common Failure Patterns

- Commands run out of order and lose context.
- Final summary omits the evidence IDs returned by commands.
- Trial matching lacks criterion-level explanation.
- Resistance and alternative-treatment claims are made without supporting queries.

## Practical Tips

- Keep raw output snippets for each checklist line item.
- Prefer explicit command reruns over inferred claims.
- Mark no-result cases clearly (for example, no recruiting trials found) rather than leaving gaps.
