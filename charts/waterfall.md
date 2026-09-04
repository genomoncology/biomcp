# Waterfall

BioMCP's waterfall chart ranks mutated samples by mutation burden for the queried gene. It is a domain-specific "waterfall" view rendered with a sorted bar chart, not Kuva's cumulative financial waterfall plot.

## Supported Commands

- `study query --type mutations --chart waterfall`

## Examples

```bash
biomcp study query --study msk_impact_2017 --gene TP53 --type mutations \
  --chart waterfall --terminal

biomcp study query --study msk_impact_2017 --gene TP53 --type mutations \
  --chart waterfall --width 1200 --height 600 -o tp53-waterfall.svg
```

## Notes

Each bar represents one mutated sample.

Samples are sorted by descending mutation count, with sample ID as the tie-breaker.
