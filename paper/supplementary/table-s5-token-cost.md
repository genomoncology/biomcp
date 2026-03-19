# Table S5. Token-cost measurements

The finished table will summarize compact-versus-naive workflow token counts, byte counts, and runtime measurements from the archived evaluation pack.

## Expected columns

| Column | Meaning |
| --- | --- |
| `Workflow` | Workflow identifier used in the paper package. |
| `Compact tokens` | Token count for the compact BioMCP workflow output. |
| `Naive tokens` | Token count for the naive baseline workflow output. |
| `Token reduction` | Percentage reduction in tokens relative to the naive baseline. |
| `Compact bytes` | Byte count for the compact workflow output. |
| `Naive bytes` | Byte count for the naive baseline output. |
| `Cold median (s)` | Median cold-start runtime in seconds. |
| `Warm median (s)` | Median warm-cache runtime in seconds. |
