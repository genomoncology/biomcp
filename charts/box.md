# Box Plot

Box plots show median, IQR, and whiskers for group comparisons.

## Supported Commands

- `study compare --type expression --chart box`

## Examples

```bash
biomcp study compare --study brca_tcga_pan_can_atlas_2018 \
  --gene TP53 --type expression --target ERBB2 \
  --chart box --terminal
```
