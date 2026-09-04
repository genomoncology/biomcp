# How to: reproduce paper workflows

BioMCP paper-style workflows are best reproduced with direct CLI commands. If
you want the agent-facing guide first, run `biomcp skill` or install it into
your agent directory, then execute the commands that match the paper pattern you
want to reproduce.

## Mapping

| Paper-style workflow | BioMCP workflow area | Representative commands |
|----------------------|----------------------|-------------------------|
| GeneGPT | gene, variant, trial, and article walkthroughs | `biomcp get gene BRAF`, `biomcp get variant "BRAF V600E" population`, `biomcp variant trials "BRAF V600E" --limit 3`, `biomcp search article -g BRAF -d melanoma --limit 3` |
| GeneAgent | pathway, drug, and protein synthesis | `biomcp get pathway R-HSA-5673001 genes`, `biomcp pathway drugs R-HSA-5673001 --limit 3`, `biomcp protein structures P15056` |
| TrialGPT | trial discovery and patient matching | `biomcp search trial -c melanoma --mutation "BRAF V600E" --status recruiting --limit 5` |
| PubMed & Beyond | literature synthesis | `biomcp search article -g BRAF -d melanoma --limit 5`, `biomcp get article 22663011 fulltext` |

## Suggested execution pattern

1. Pick the workflow area that matches the paper task.
2. Run the direct CLI commands for that area.
3. Save command output in a markdown log with stable IDs such as PMIDs, gene
   symbols, pathway IDs, and NCT IDs.
4. Verify the final summary against the paper's reported entities and evidence.

## Example session

```bash
biomcp get gene BRAF
biomcp get variant "BRAF V600E" population
biomcp search trial -c melanoma --mutation "BRAF V600E" --status recruiting --limit 5
biomcp get article 22663011 fulltext
```

## Notes

- The BioMCP guide is optional context, not the primary execution path.
- If a service is temporarily rate limited, retry after a short pause.
- If enrichment is unavailable, continue with pathway, interaction, or
  literature checks that answer the same paper question.
