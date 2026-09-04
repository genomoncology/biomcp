# Density Plot (KDE)

Density plots use kernel density estimation for a smooth view of expression distributions.

## Supported Commands

- `study query --type expression --chart density`

## Examples

```bash
biomcp study query --study brca_tcga_pan_can_atlas_2018 --gene ERBB2 --type expression \
  --chart density --terminal
```
