# Histogram

Histograms show the distribution of continuous values as binned counts. Use for single-gene expression distributions.

## Supported Commands

- `study query --type expression --chart histogram`

## Examples

```bash
# ERBB2 expression histogram in terminal
biomcp study query --study brca_tcga_pan_can_atlas_2018 --gene ERBB2 --type expression \
  --chart histogram --terminal

# Write to SVG
biomcp study query --study brca_tcga_pan_can_atlas_2018 --gene ERBB2 --type expression \
  --chart histogram -o erbb2-expr.svg
```

## Notes

Bins default to 30. The histogram reads raw expression values from the study matrix file.
