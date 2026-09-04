# Violin Plot

Violin plots show full distribution shapes for group comparisons, combining a box plot with a kernel density estimate.

## Supported Commands

- `study compare --type expression --chart violin`

## Examples

```bash
biomcp study compare --study brca_tcga_pan_can_atlas_2018 \
  --gene TP53 --type expression --target ERBB2 \
  --chart violin --terminal

biomcp study compare --study brca_tcga_pan_can_atlas_2018 \
  --gene TP53 --type expression --target ERBB2 \
  --chart violin -o erbb2-by-tp53.svg
```
