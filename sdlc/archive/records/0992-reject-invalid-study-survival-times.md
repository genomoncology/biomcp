---
flow: quickfix
priority: 10
---

# Reject invalid study survival times

Local study survival analysis currently counts negative and non-finite month values as biomedical observations. Treat those cells as missing before Kaplan–Meier calculation. Zero and every finite nonnegative value remain valid, and neighboring valid rows must still contribute normally.

Red-green coverage belongs in `src/sources/cbioportal_study.rs` and `spec/entity/study.md`; existing survival expectations there may be restated to express the corrected input contract.
