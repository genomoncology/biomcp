# Bar Chart

Bar charts display categorical counts as vertical bars. Use for mutation variant classes, CNA bucket distributions, or co-occurrence category sizes.

## Supported Commands

- `study query --type mutations --chart bar`
- `study query --type cna --chart bar`
- `study co-occurrence --chart bar`
- `study compare --type mutations --chart bar`
- `study survival --chart bar`

## Examples

```bash
# Mutation variant class distribution
biomcp study query --study msk_impact_2017 --gene TP53 --type mutations \
  --chart bar --terminal

# CNA distribution as SVG
biomcp study query --study brca_tcga_pan_can_atlas_2018 --gene ERBB2 --type cna \
  --chart bar -o erbb2-cna.svg

# Survival event rates per group
biomcp study survival --study brca_tcga_pan_can_atlas_2018 --gene TP53 \
  --chart bar --terminal
```
