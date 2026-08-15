---
flow: build
priority: 10
---

# 0989: Make release evidence truthful before publication

## Outcome

A release can publish only after BioMCP has inspected the exact assembled
packages, executed both commands installed from every wheel, and validated all
required manual promotion evidence; the GitHub release body comes from the
curated changelog rather than unrelated generated notes.

## Current facts

`release/build_target.py` currently writes `"inspected": true` into native and
wheel records after inspecting only the raw executable's platform headers. It
does not invoke the existing assembled-package inspection in
`release/inspect.py`. The public platform job installs each wheel but executes
only the separately downloaded native binary. The promotion workflow validates
the Windows Claude Desktop and updater-transition inputs after immutable public
writes. `release/publish-versioned.sh` creates GitHub releases with
`--generate-notes`, which can disagree with the curated `CHANGELOG.md` entry.

## Scope

- Make the assembled-artifact inspector the single final writer of artifact
  evidence. It must validate and retain the platform-floor result, SBOM hash,
  binary hash, applicable shim hash, and signing evidence while adding the
  archive member checks, executable checks, identity-aware smoke result, and
  exact assembled-artifact hash. Do not predeclare inspection success, and do
  not leave a success record when inspection fails.
- In each of the five private target assembly jobs, before `seal-candidate`,
  install that job's final wheel into an isolated environment and run
  identity-aware `release/smoke.py` checks against both installed commands:
  `biomcp` and the `biomcp-cli` compatibility command. Post-public checks may
  repeat this proof but cannot substitute for the private pre-public gate.
- Validate the exact Windows Desktop and updater-transition inputs during
  promotion preflight, before `publish-versioned` can run. Store their
  normalized validated values, or canonical hashes plus the normalized updater
  result, in `promotion-inventory.json`. Publication and reconciliation must
  consume that bound inventory rather than independently trusting the original
  workflow strings.
- Extract the exact version's curated changelog section into a release-notes
  file and pass that file to GitHub release creation. Reject a missing,
  duplicate, empty, or mismatched version section; do not fall back to generated
  notes.

The existing package tests in `tests/test_release_package.py`, target-build
tests in `tests/test_release_targets.py`, promotion tests in
`tests/test_release_promotion.py`, and workflow assertions in
`tests/test_release_stage_workflow.py` may be extended or restated. Add a
focused release-notes test next to the owning release script.

This ticket changes release construction and promotion evidence only. It does
not publish a release, change versions, enable signing, or modify changelog
content.

## Acceptance

- A malformed assembled archive or wheel leaves no success record. Valid native
  evidence contains the exact artifact hash, archive/executable counts,
  identity-aware smoke, platform floor, SBOM hash, binary hash, and signing
  result when signing applies. Valid wheel evidence contains those applicable
  fields plus its shim hash and the validated small-shim result.
- Workflow contracts prove both wheel-installed commands pass identity-aware
  smoke in all five private platform jobs before candidate sealing.
- A missing or invalid Windows Desktop or updater record stops
  `promotion-preflight`. The resulting inventory binds the normalized evidence
  (or canonical hashes and updater result), and publication/reconciliation
  consume those bound values instead of the raw workflow inputs.
- GitHub release creation uses only the exact curated changelog section for the
  candidate version and contains no `--generate-notes` fallback.
- Focused package, target, promotion, release-script, and workflow tests pass,
  followed by `make lint` and `git diff --check`.

## Dependencies

Ticket 0988, because this work relies on trustworthy workflow checks.

## Review

- Design review: accepted after making pre-public execution order, final
  evidence ownership, and normalized manual-evidence binding explicit
- Code review: accepted after independently revalidating retained evidence,
  binding the actual latest public release, fully checking wheels, and requiring
  exact structured binary identity
