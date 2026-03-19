# BioMCP Charts

BioMCP provides native chart output for study commands via the `--chart` flag. Charts render to your terminal using Unicode block characters, or to SVG or PNG files.

## Quick Start

```bash
# Terminal bar chart of TP53 mutation types
biomcp study query --study msk_impact_2017 --gene TP53 --type mutations \
  --chart bar --terminal

# SVG pie chart of co-occurrence proportions
biomcp study co-occurrence --study msk_impact_2017 --genes TP53,KRAS \
  --chart pie -o tp53-kras.svg

# SVG Kaplan-Meier curve by TP53 mutation status
biomcp study survival --study brca_tcga_pan_can_atlas_2018 --gene TP53 \
  --chart survival -o tp53-km.svg

# Terminal violin plot of ERBB2 expression by TP53 status
biomcp study compare --study brca_tcga_pan_can_atlas_2018 \
  --gene TP53 --type expression --target ERBB2 \
  --chart violin --terminal
```

## Chart Types by Command

| Command | Valid Chart Types |
|---------|------------------|
| `study query --type mutations` | `bar`, `pie` |
| `study query --type cna` | `bar`, `pie` |
| `study query --type expression` | `histogram`, `density` |
| `study co-occurrence` | `pie`, `bar` |
| `study compare --type expression` | `box`, `violin`, `ridgeline` |
| `study compare --type mutations` | `bar` |
| `study survival` | `bar`, `survival` |

Invalid combinations return an error listing the valid options.

## Output Formats

| Flag | Effect |
|------|--------|
| `--terminal` | Render to terminal (Unicode block characters) |
| `-o file.svg` | Write SVG (exact data recovery, AI-readable) |
| `-o file.png` | Write PNG (requires `--features charts-png` build) |

When `--chart` is specified without `--terminal` or `-o`, BioMCP defaults to `--terminal`.

## Styling Options

| Flag | Values | Default |
|------|--------|---------|
| `--title TEXT` | Any string | Auto-generated |
| `--theme NAME` | `light`, `dark`, `solarized`, `minimal` | terminal: `dark`; files: `light` |
| `--palette NAME` | `category10`, `wong`, `okabe-ito`, `tol-bright`, `tol-muted`, `tol-light`, `ibm`, `deuteranopia`, `protanopia`, `tritanopia`, `pastel`, `bold` | `category10` |

Use `--palette wong` for colorblind-safe output.

## Why SVG?

SVG is the recommended format for AI-assisted workflows. An AI agent can parse SVG XML attributes to recover exact numeric values (100% accuracy), compared to ~97% for PNG and ~90% for terminal. SVG is also 46x smaller than equivalent PNG.

```xml
<!-- AI reads height="405.9" and scale to recover the exact count: 3157 -->
<rect height="405.9" .../>
```

## Chart Reference Pages

- [`biomcp chart bar`](bar.md) — Bar chart
- [`biomcp chart pie`](pie.md) — Pie chart
- [`biomcp chart histogram`](histogram.md) — Histogram
- [`biomcp chart density`](density.md) — Density (KDE)
- [`biomcp chart box`](box.md) — Box plot
- [`biomcp chart violin`](violin.md) — Violin plot
- [`biomcp chart ridgeline`](ridgeline.md) — Ridgeline plot
- [`biomcp chart survival`](survival.md) — Kaplan-Meier survival curve
