# Scatter

Scatter plots render paired expression values for two genes across the same study samples.

## Supported Commands

- `study compare --type expression --chart scatter`

## Examples

```bash
biomcp study compare --study brca_tcga_pan_can_atlas_2018 \
  --gene TP53 --type expression --target ERBB2 \
  --chart scatter --terminal

biomcp study compare --study brca_tcga_pan_can_atlas_2018 \
  --gene TP53 --type expression --target ERBB2 \
  --chart scatter --width 1200 --height 600 -o tp53-vs-erbb2.svg
```

## Notes

`--gene` is the x-axis gene and `--target` is the y-axis gene.

Only samples with numeric expression values for both genes are plotted.
