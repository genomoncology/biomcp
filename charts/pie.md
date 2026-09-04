# Pie Chart

Pie charts display proportional distributions. Especially useful for co-occurrence categories (both mutated, gene A only, gene B only, neither).

## Supported Commands

- `study query --type mutations --chart pie`
- `study query --type cna --chart pie`
- `study co-occurrence --chart pie`

## Examples

```bash
# TP53/KRAS co-occurrence proportions
biomcp study co-occurrence --study msk_impact_2017 --genes TP53,KRAS \
  --chart pie --terminal

# Mutation variant class pie
biomcp study query --study msk_impact_2017 --gene TP53 --type mutations \
  --chart pie -o tp53-variants.svg
```
