# Ridgeline Plot

Ridgeline plots stack density estimates per group, making it easy to compare distributions across many groups.

## Supported Commands

- `study compare --type expression --chart ridgeline`

## Examples

```bash
biomcp study compare --study brca_tcga_pan_can_atlas_2018 \
  --gene TP53 --type expression --target ERBB2 \
  --chart ridgeline --terminal
```
