---
priority: 2
---

# 2288 — Run the installed-wheel smoke on a fresh home in the routine check

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 9da712f, accept

## Outcome

The routine full main check builds the Linux wheel, installs it in the same manylinux container release.yml uses, and runs the release.yml installed-wheel smoke commands under a fresh HOME with no cache ancestors. It uses the same `BIOMCP_DISABLE_SOURCES` profile as release.yml. A fresh-root, packaging or installed-command failure fails the routine check, so the GitHub release rehearsal is never the first place one appears.

Changes: `Makefile`, `.github/workflows/release.yml`, `scripts/installed-wheel-smoke.sh`, `scripts/check-linux-wheel.sh`, `tests/test_release_workflow_provenance.py`, `tests/test_installed_wheel_smoke.py`, `sdlc/records/2288-local-installed-wheel-check.md`, `CHANGELOG.md`, `tests/test_routine_cargo_feature_contract.py`.

- Proof: Breaking fresh-root directory creation on a branch fails the routine check. The routine check and the release rehearsal run the same smoke command list.

## Evidence

In 0.9.4 the first full rehearsal found three problems no local check had run: installed Linux wheels failed `cache stats --json` on a fresh root (2286), the DDInter smoke could not pass under the commercial-restricted profile, and the article smoke called Semantic Scholar through enrichment. Each cost a rehearsal and a new qualification round.

Reuse the release.yml smoke command list from one shared source, so the local and release smokes cannot drift. Run serially with the existing caps and pacing. Use no keys. Keep KEGG and Semantic Scholar uncalled. Where a public provider call is needed, keep it to the commands release.yml already runs.

## Implementation boundary

Use one shared smoke command list with fresh home and keyless environment on native and manylinux execution. The local wrapper reads the release matrix image source; pin the two existing Linux build/smoke images to the exact digests already qualified in 2286. Add checkout before using shared scripts. Fakes must make cold-root/cache and other unexpected product failures fatal and print captured stdout and stderr before any provider call. Preserve approved expected-disabled responses.

Keep: experiments/2102-biomcp-2258-final-live-replay — retain installed-wheel-process/2288 raw source, fake controls, failed negative-control harness output and actual-main artifact proof until release admission closes. Original wheel/image identities remain in experiment 492.

Landing alone: this changes the main gate and release workflow, so its gate definition must land separately. Source and negative controls are reviewed; the actual-main full gate supplies the single positive current-artifact smoke before another ordinary landing.
