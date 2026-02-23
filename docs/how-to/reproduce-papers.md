# Reproduce Paper Workflows with Skills

BioMCP paper demos are consolidated into embedded skill workflows so any agent can run them without a custom runtime. Use this guide to map each paper-style task to the canonical skill and execution pattern.

## Mapping

| Paper-style workflow | Canonical skill | Typical command |
|----------------------|-----------------|-----------------|
| GeneGPT | `09-gene-function-lookup` | `biomcp skill gene-function-lookup` |
| GeneAgent | `10-gene-set-analysis` | `biomcp skill gene-set-analysis` |
| TrialGPT | `03-trial-searching` (patient matching section) | `biomcp skill 03` |
| PubMed & Beyond | `11-literature-synthesis` | `biomcp skill literature-synthesis` |

## Basic Flow

1. List available workflows.
2. Retrieve one by number or slug.
3. Execute the included CLI steps.
4. Verify results against that skill's validation checklist.

```bash
biomcp skill list
biomcp skill gene-function-lookup
```

## Suggested Execution Pattern

For a reproducible run, copy command output to a markdown log and keep IDs (PMIDs/NCT IDs) visible in the final summary.

```bash
biomcp skill 10
biomcp enrich "BRCA1,TP53,ATM,CHEK2,PALB2" --limit 10
biomcp get gene BRCA1 pathways
```

## Notes

- Skills are model agnostic; any shell-capable agent can execute them.
- If a service is temporarily rate limited, retry after a short pause.
- If enrichment is unavailable, continue with pathway/interaction/literature verification steps in skill 10.
