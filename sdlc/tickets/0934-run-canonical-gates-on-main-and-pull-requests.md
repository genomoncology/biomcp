---
flow: build
priority: 9
deps: ["0951"]
---
# Run canonical gates on main and pull requests

The hosted CI workflow runs only for pull requests, while this repository's
factory normally lands directly on `main`. Its hand-written Rust commands also
omit parts of the canonical lint gate. Contract and release workflows do not
protect an ordinary main push.

## Integration contract

A required hosted workflow runs for pull requests and every push to `main`.
It invokes the repository's canonical `make lint`, `make test`, and `make spec`
contracts without copying a smaller approximation into YAML. Jobs may split
those targets for useful failure reporting only when a checked wrapper proves
their combined command set is exactly the canonical contract.

Install the pinned Rust, Python/uv, Ruff, cargo-deny, protoc, and other required
gate tools. A missing required tool fails; it never turns a check into a skip.
Routine CI uses committed/local fixtures and no credentials or public provider
calls. Windows-specific source contracts remain covered by their existing job.
Release publication stays outside this workflow.

## Done when

- Workflow-trigger tests prove both `pull_request` and push-to-main coverage.
- A workflow contract proves canonical target invocation and fails if a target
  is replaced by an incomplete command list.
- The spec job identifies the artifact it actually builds and uses; comments
  no longer claim a release binary while running the spec-profile binary.
- Required tools and action versions are explicit and reproducible.
- Documentation-only optimization remains owned by ticket 0895 and may not
  skip a required non-Rust documentation contract.
- Architecture/operator CI documentation is checked against the real jobs and
  triggers rather than a copied job count.

## Authorized test changes

Design commits may restate `.github/workflows/ci.yml`, CI contract tests,
canonical-gate wrappers, and CI architecture/operator documentation. Existing
Windows CSpec, MCP contract, wheel smoke, and live-provider separation remain
covered. No product source change belongs here.

The src line ceiling may not rise.
