---
base: 6c3ab2b32ad7f4cb0b26a490a4f6a89c153adc1f
head: a7dde1d2560cdf248a316180640bd10368c9a285
---
BioMCP hardcodes terminal chart dimensions at 100x32 characters and SVG/PNG at Kuva's defaults. Users can't control chart size for presentations, papers, or slide assets. The underlying Kuva library fully supports arbitrary dimensions — BioMCP just doesn't expose the knobs.

Imported from March ticket 062. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/062-chart-dimensions-and-new-types
