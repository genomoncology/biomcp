---
flow: build
priority: 6
---
# Chart dimensions, DPI control, and new chart types

BioMCP hardcodes terminal chart dimensions at 100x32 characters and SVG/PNG at Kuva's defaults. Users can't control chart size for presentations, papers, or slide assets. The underlying Kuva library fully supports arbitrary dimensions — BioMCP just doesn't expose the knobs.

Completed under March on 2026-03-26, as March ticket 062. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/062-chart-dimensions-and-new-types
