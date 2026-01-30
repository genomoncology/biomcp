# Use Case: Rare Disease Research

Find clinical trials and research for rare diseases with limited treatment options.

## Workflow

1. **Search clinical trials** - Find recruiting trials for the condition
2. **Filter by intervention type** - Gene therapy, small molecule, etc.
3. **Get disease information** - Ontology terms and synonyms
4. **Search literature** - Recent therapeutic research
5. **Check causative genes** - Variants in known disease genes

## Rare Disease Trial Strategies

| Strategy        | Filter                        |
| --------------- | ----------------------------- |
| Gene therapy    | `--intervention-type genetic` |
| Natural history | `--type observational`        |
| Expanded access | `--type expanded_access`      |
| Pediatric       | `--age-group child`           |

## Tips

- Rare diseases often have few trials - use broad searches
- Check for patient registries (`--type observational`)
- Gene therapy trials are often early phase
- Use disease synonyms for comprehensive coverage
