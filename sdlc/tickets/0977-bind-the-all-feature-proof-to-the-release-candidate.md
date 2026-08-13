---
flow: build
priority: 10
---
# Bind the all-feature proof to the release candidate

Routine development gates intentionally compile a small
`--no-default-features` graph. The repository says `make full-feature-check`
is the proof for all shipped features, including AlphaGenome behavior and an
all-feature release build, but the release workflow records only `lint`,
`test`, and `spec` in the candidate manifest. Artifact jobs compile shipped
features without running the omitted tests or Clippy, and a separate CI job is
not bound to the candidate evidence for the selected commit.

A candidate must not become complete unless the all-feature proof for its exact
source commit passed. Run that proof during staging and record it in the sealed
manifest. Do not infer success merely because an unbound CI workflow may have
run on the same branch.

## Done when

- The release workflow runs the repository's declared all-feature proof for
  the selected source commit before candidate initialization completes.
- `full-feature-check` is a required, recorded candidate gate; omitting it or
  changing its recorded result prevents finalization and promotion.
- Candidate tests prove that the old three-gate manifest is insufficient.
- Developer documentation, the local release gate, CI, and release staging
  name the same small routine lane and the same additional shipped-feature
  proof.

## Authorized test changes

Design may restate candidate-gate assertions in `tests/test_release_candidate.py`,
release workflow assertions in `tests/test_release_stage_workflow.py`,
`tests/test_release_workflow_provenance.py`, and
`tests/test_routine_cargo_feature_contract.py`, and the corresponding release
gate assertions in `tests/surface/test_parallel_isolation_contract.py`.
