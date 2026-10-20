---
base: a462cdd3784f658ea0c1820cfbc242f2abcb00ec
head: 9c1d5d587d8292613485678b2e8bf514414ee2df
---

# Fail fast when a CI step stalls

All 24 jobs in the CI, contract, and release workflows now have a 45-minute
job timeout. This bounds stalled dependency installation and later work while
leaving the existing acquisition paths and gates unchanged. The limit gives
headroom over the observed 25-minute canonical, 13-minute full-feature, and
7-minute Windows runs.
