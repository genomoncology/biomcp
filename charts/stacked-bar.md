# Stacked Bar Chart

Stacked bar charts split each mutation-defined group into mutated and not-mutated sample counts.

## Supported Commands

- `study compare --type mutations --chart stacked-bar`

## Examples

```bash
biomcp study compare --study brca_tcga_pan_can_atlas_2018 \
  --gene TP53 --type mutations --target PIK3CA \
  --chart stacked-bar --terminal

biomcp study compare --study brca_tcga_pan_can_atlas_2018 \
  --gene TP53 --type mutations --target PIK3CA \
  --chart stacked-bar -o pik3ca-by-tp53-stacked.svg
```

## Notes

`--chart stacked-bar` shows sample counts. `--chart bar` on the same command continues to show mutation rate by group.
