# Pattern: Resistance & Next-Line Options

Research resistance mechanisms and find trials for patients who progressed on prior therapy.

## Workflow

1. **Search resistance literature** - `biomcp article search -g <gene> -c <drug> -k resistance`
2. **Find acquired mutations** - `biomcp variant search --gene <gene>` for known resistance variants
3. **Find post-progression trials** - `biomcp trial search` with prior therapy filters
4. **Check alternative agents** - `biomcp drug search` for other drugs targeting the pathway

## Search Strategies

| Goal                  | Approach                                       |
| --------------------- | ---------------------------------------------- |
| Resistance mechanisms | `-k resistance` or `-k refractory` in articles |
| Second-line trials    | `--line-of-therapy 2L` in trial search         |
| Prior treatment       | `--prior-therapy <drug>` in trial search       |

## Common Resistance Patterns

| Original Therapy   | Gene    | Resistance Mutation |
| ------------------ | ------- | ------------------- |
| EGFR TKI (1st gen) | EGFR    | T790M               |
| Osimertinib        | EGFR    | C797S               |
| BRAF inhibitor     | BRAF    | Splice variants     |
| Imatinib           | BCR-ABL | T315I               |

## Tips

- Combine gene + drug + "resistance" for targeted literature
- Check if resistance mutation has its own targeted therapy
- Look for trials specifically enrolling patients with acquired resistance
