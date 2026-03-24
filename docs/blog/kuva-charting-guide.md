# Charts Your Agent Can Actually Read

*Most charting libraries are built for human eyes. BioMCP's are built for both.*

![Agent Charting Superpowers](images/kuva-charting-slide.png)

When an AI agent generates a PNG chart, it can't read its own output. It rendered pixels it has no way to parse back into numbers. The agent needs a vision model, a screenshot, and a prayer.

BioMCP takes a different approach. Every study command that produces data can also produce a chart — in three formats, each designed for a different consumer:

- **Terminal** renders the chart inline using Unicode block and Braille characters. The agent sees it in its own context window. No file, no screenshot, no round-trip.
- **SVG** is structured XML. An agent can parse exact numeric values directly from element attributes — 100% accuracy, no vision model needed, 36x smaller than PNG.
- **PNG** is for humans sharing charts in presentations and social media.

The charting engine is [Kuva](https://github.com/Psy-Fer/kuva), an open-source Rust library with 29 plot types. BioMCP links it directly — no Python, no R, no subprocess. Charts compile into the single binary.

## Why this matters for agents

A coding agent running BioMCP in a terminal session can query data *and* see the chart in the same output stream. It doesn't need to open a file, switch windows, or invoke a vision model.

```bash
biomcp study survival --study msk_impact_2017 --gene TP53 \
  --chart survival --terminal
```

![Terminal survival chart](images/kuva-terminal-survival.png)

That's a Kaplan-Meier survival curve drawn with Braille characters directly in the terminal. The agent can see the separation between TP53-mutant and wildtype groups, reason about the pattern, and immediately run the next command.

When the agent needs precision, SVG is the better output:

```xml
<rect x="59" y="88.1" width="68.57" height="405.9" fill="#1f77b4"/>
```

From `height="405.9"` and the axis scale, the agent recovers the exact count: 3,157 missense mutations. No estimation. No OCR. The TP53 mutation bar chart is 4.9 KB as SVG versus ~175 KB as PNG — **36x smaller** with higher fidelity.

## Eight chart types

Add `--chart <type>` to any study command. BioMCP validates the combination — ask for a violin plot from a mutation query and it tells you what's valid instead of producing garbage.

### Survival curves

```bash
biomcp study survival --study msk_impact_2017 --gene TP53 \
  --chart survival -o tp53-survival.svg
```

![TP53 survival](images/kuva-tp53-survival-km.svg)

TP53-mutant patients: median 21.0 months. Wildtype: 32.1 months. p = 9.40e-29.

### Expression distributions

```bash
biomcp study query --study brca_tcga_pan_can_atlas_2018 --gene ERBB2 \
  --type expression --chart histogram -o erbb2-histogram.svg
```

![ERBB2 histogram](images/kuva-erbb2-expression-histogram.svg)

The bimodal distribution is the signature of HER2-positive breast cancer. The right-hand bump is the ~15-20% of breast cancers with HER2 amplification.

### Violin and ridgeline plots

```bash
biomcp study compare --study brca_tcga_pan_can_atlas_2018 \
  --gene TP53 --type expression --target ERBB2 \
  --chart violin -o erbb2-by-tp53-violin.svg
```

![ERBB2 violin](images/kuva-erbb2-by-tp53-violin.svg)

ERBB2 expression stratified by TP53 mutation status. The violin reveals the full distribution shape — the bimodal HER2 pattern is visible in both groups.

![ERBB2 ridgeline](images/kuva-erbb2-by-tp53-ridgeline.svg)

Ridgelines stack the same density curves vertically for easier visual comparison.

### Mutation and co-occurrence charts

```bash
biomcp study query --study msk_impact_2017 --gene TP53 --type mutations \
  --chart bar -o tp53-mutation-bar.svg
```

![TP53 mutations bar](images/kuva-tp53-mutation-bar.svg)

![Terminal bar chart](images/kuva-terminal-bar.png)

3,157 missense mutations dominate TP53 in MSK-IMPACT, followed by 683 nonsense and 517 frameshift deletions. Same data, two renderings — SVG for sharing, terminal for exploration.

## Themes and accessibility

Four themes and twelve color palettes, including five designed for colorblind accessibility:

```bash
biomcp study query --study msk_impact_2017 --gene TP53 --type mutations \
  --chart bar --theme dark --palette wong
```

| Themes | Accessible palettes |
|--------|-------------------|
| `light`, `dark`, `solarized`, `minimal` | `wong`, `okabe-ito`, `deuteranopia`, `protanopia`, `tritanopia` |

## What's next

BioMCP currently uses 8 of Kuva's 29 chart types. Heatmaps for co-occurrence matrices, scatter plots for two-gene expression comparisons, and waterfall plots for mutation burden are on the roadmap.

## Try it

```bash
uv tool install biomcp-cli
biomcp study download msk_impact_2017
biomcp study survival --study msk_impact_2017 --gene TP53 \
  --chart survival --terminal
```

Full chart reference: [Chart Documentation](../charts/index.md).
