---
flow: quickfix
priority: 10
---

# Bound chart dimensions

Reject unusably small or excessive chart dimensions before study loading. Terminal charts allow 20–500 columns and 8–200 rows. SVG, PNG, and inline MCP charts allow width 240–4096 and height 160–4096. PNG scale is 0.5–4.0 and the final checked pixel count cannot exceed 16,777,216.

Red-green coverage belongs in `src/cli/study/tests/charts.rs`, `src/cli/tests/facade/chart.rs`, and the chart CLI/MCP process contracts; existing permissive boundary assertions may be restated.
