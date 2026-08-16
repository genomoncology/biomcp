---
flow: quickfix
priority: 10
---

# Treat a missing study root as an empty catalog

A fresh installation with no local cBioPortal study directory must behave like an empty catalog. `study list` succeeds empty, and a query returns the normal `not_in_local_cohorts` result and download guidance. A path that exists but is not a usable directory remains an error.

Red-green coverage belongs in `src/cli/study/tests.rs`, `src/sources/cbioportal_study.rs`, and `spec/entity/study.md`; those files may restate the former missing-root failure.
