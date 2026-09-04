# BioMCP Charts

BioMCP provides native chart output for study commands via the `--chart` flag. Charts render to your terminal using Unicode block characters, or to SVG and PNG when you need a durable export.

## Quick Start

```bash
# Terminal waterfall chart with custom character-cell dimensions
biomcp study query --study msk_impact_2017 --gene TP53 --type mutations \
  --chart waterfall --cols 80 --rows 24

# SVG histogram with an explicit canvas size
biomcp study query --study brca_tcga_pan_can_atlas_2018 --gene ERBB2 \
  --type expression --chart histogram \
  --width 1200 --height 600 -o erbb2-histogram.svg

# Scatter plot of paired expression values
biomcp study compare --study brca_tcga_pan_can_atlas_2018 \
  --gene TP53 --type expression --target ERBB2 \
  --chart scatter --terminal

# PNG survival curve at higher pixel density
biomcp study survival --study brca_tcga_pan_can_atlas_2018 --gene TP53 \
  --chart survival --scale 3.0 -o tp53-km.png
```

## Chart Types by Command

| Command | Valid Chart Types |
|---------|------------------|
| `study query --type mutations` | `bar`, `pie`, `waterfall` |
| `study query --type cna` | `bar`, `pie` |
| `study query --type expression` | `histogram`, `density` |
| `study co-occurrence` | `bar`, `pie`, `heatmap` |
| `study compare --type expression` | `box`, `violin`, `ridgeline`, `scatter` |
| `study compare --type mutations` | `bar`, `stacked-bar` |
| `study survival` | `bar`, `survival` |

Invalid combinations return an error listing the valid options for that command and data shape.

## Output Formats

| Target | How to select it | Supported size flags |
|--------|------------------|----------------------|
| Terminal | `--chart ...` with no file output, or `--terminal` | `--cols`, `--rows` |
| SVG file | `-o file.svg` | `--width`, `--height` |
| PNG file | `-o file.png` | `--width`, `--height`, `--scale` |
| MCP inline SVG | MCP chart responses | `--width`, `--height` |

When `--chart` is specified without `--terminal` or `-o`, BioMCP defaults to terminal rendering.

## Styling and Size Flags

| Flag | Applies to | Default |
|------|------------|---------|
| `--title TEXT` | All chart outputs | Auto-generated title |
| `--theme NAME` | All chart outputs | terminal: `dark`; files/inline SVG: `light` |
| `--palette NAME` | Categorical charts | `category10` |
| `--cols N` | Terminal charts only; 20–500 | `100` |
| `--rows N` | Terminal charts only; 8–200 | `32` |
| `--width PX` | SVG, PNG, MCP inline SVG; 240–4096 | Kuva auto layout |
| `--height PX` | SVG, PNG, MCP inline SVG; 160–4096 | Kuva auto layout |
| `--scale FACTOR` | PNG only; 0.5–4.0 | `2.0` |

PNG output is additionally limited to 16,777,216 final pixels after applying
the scale factor. Invalid bounds fail before BioMCP loads local study data.

Use `--palette wong` for colorblind-safe categorical output.

Heatmaps use a fixed continuous colormap. `study co-occurrence --chart heatmap` supports `--title`, `--theme`, `--width`, and `--height`, but rejects `--palette`.

## Why SVG?

SVG is the recommended format for AI-assisted workflows. An AI agent can parse SVG XML attributes to recover exact numeric values, while terminal output is optimized for fast exploration and PNG is optimized for sharing in human-facing assets.

## Chart Reference Pages

- [`biomcp chart bar`](bar.md) — Bar chart
- [`biomcp chart stacked-bar`](stacked-bar.md) — Stacked bar chart
- [`biomcp chart pie`](pie.md) — Pie chart
- [`biomcp chart waterfall`](waterfall.md) — Ranked mutation-burden waterfall
- [`biomcp chart heatmap`](heatmap.md) — Heatmap
- [`biomcp chart histogram`](histogram.md) — Histogram
- [`biomcp chart density`](density.md) — Density (KDE)
- [`biomcp chart box`](box.md) — Box plot
- [`biomcp chart violin`](violin.md) — Violin plot
- [`biomcp chart ridgeline`](ridgeline.md) — Ridgeline plot
- [`biomcp chart scatter`](scatter.md) — Scatter plot
- [`biomcp chart survival`](survival.md) — Kaplan-Meier survival curve
