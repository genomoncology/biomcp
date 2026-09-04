# Heatmap

Heatmaps render pairwise co-mutation counts as an NxN matrix so larger co-occurrence sets stay readable.

## Supported Commands

- `study co-occurrence --chart heatmap`

## Examples

```bash
biomcp study co-occurrence --study msk_impact_2017 \
  --genes TP53,KRAS,PIK3CA,EGFR --chart heatmap --terminal

biomcp study co-occurrence --study msk_impact_2017 \
  --genes TP53,KRAS,PIK3CA,EGFR --chart heatmap -o cooccurrence-heatmap.svg
```

## Notes

Diagonal cells are placeholders rather than self-co-occurrence measurements.

`--palette` is not supported for heatmaps in this release. Use `--theme` and `--title`; BioMCP applies a fixed continuous colormap to the cells.
