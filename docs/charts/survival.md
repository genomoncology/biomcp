# Survival Chart

Survival charts render Kaplan-Meier step curves for mutation-defined study cohorts.

## Supported Commands

- `study survival --chart survival`

## Output Modes

- `--terminal` for an in-terminal step plot
- `-o file.svg` for SVG output
- `-o file.png` for PNG output when BioMCP is built with `--features charts-png`

## Examples

```bash
# Terminal Kaplan-Meier plot
biomcp study survival --study brca_tcga_pan_can_atlas_2018 --gene TP53 \
  --chart survival --terminal

# SVG Kaplan-Meier plot
biomcp study survival --study brca_tcga_pan_can_atlas_2018 --gene TP53 \
  --endpoint os --chart survival -o tp53-km.svg
```
