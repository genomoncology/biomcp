---
base: 0780dff7dd086826b4eff1a9827158dfe00e351e
head: b5e6ef4acb3baa9722e624a85f50deece957a179
---

# NCI trial details use provider field locations

NCI detail conversion now reads interventions, age bounds, study type,
enrollment, and stop reason from the recorded provider shape. This restores
values that were silently empty or wrong while preserving checked stop-reason
absence.
