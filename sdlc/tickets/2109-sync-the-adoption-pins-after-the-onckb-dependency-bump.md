---
flow: build
priority: 1
deps: [2098, 2041]
---

# 2109: Sync the adoption pins after the OncoKB dependency bump

Status: OPEN.

Milestone: 1.0

## Evidence

- The 1.0 line's Cargo.toml bumped biodata to rev c46b3e6cfaf9659a8add076734b44afb4bccd002 on 2026-10-07 (09853bed1, Adopt the shared OncoKB annotation response) without updating the adoption pin chain: tests/test_biodata_model_reference.py and website/generate.py still pin ad6219d2e61ed208a8bf6ac4caca1131c4a4e200, and website/biodata-adoption*.json still record the old revision and bundle digests. website/generate.py --check fails with "BioData dependency does not match adoption manifest".
- CI surfaced it on the first line-tree branch pushed to CI (2098's, run 38818793860 repository-contracts): the line branch itself has no CI runs. Separately that run's build job failed on an unresolvable actions/setup-node pin (1e60f620b9541d33f31e4d850c561a0a7a76a4f4) carried in the line's ci.yml.
- 2098's landing is held behind this ticket: its branch is reviewed (85164a5a2) and CI-green except for these two pre-existing line defects.

## Work

- Update the adoption pin chain to c46b3e6 (test constant, generator, website manifests) and regenerate the website outputs against that revision, following the 2026-10-03 sync precedent (129b343f3).
- Fix the setup-node action pin in .github/workflows/ci.yml on the line to a resolvable ref.
- Land on the 1.0 line, then rebase 2098's branch and re-run its CI.

## Acceptance

1. website/generate.py --check passes on the line and 2098's rebased branch; CI on 2098's branch is green in every lane.
