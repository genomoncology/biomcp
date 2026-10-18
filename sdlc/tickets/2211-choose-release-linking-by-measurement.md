---
priority: 3
---

# 2211 — Choose release linking by measurement

Status: OPEN.

Milestone: 0.9.4

## Outcome

The release build profile's link settings are chosen from measured release build time and measured program speed, under the standing rule that program speed matters more than build speed.

- Evidence: the 0.9.3 dependency design (step 10) asked to measure codegen units and link-time optimization before deciding; 2116 only described the current profile.
- Starts from: the current release profile; compare it with at least one cheaper setting.
- Proof: release build time, binary size and the existing runtime cases for each setting on one build machine; keep the current profile unless another is no slower at runtime.
