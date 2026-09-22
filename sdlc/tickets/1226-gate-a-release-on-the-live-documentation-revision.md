---
flow: build
priority: 3
deps: [1222]
---

# 1226: Gate a release on the live documentation revision

## Goal

A release stops publishing while biomcp.org serves documentation older than the tag. Today the release path has no such check, and the one witness file the site keeps disappears as soon as main advances, so a naive equality check would both miss stale sites and block correct releases.

## Current Facts

- `scripts/copy-markdown-twins.py:26` removes the witness directory and writes exactly one revision file per deploy; `tests/test_docs_publication_contract.py:126-136` asserts that only the newest file survives.
- The docs site deploys main's tip (`docs-edge.yml`), and `docs/reference/release-process.md:84-85` records that the site tracks main, not the latest tag.
- Record 1224 observed the live witness returning main's tip (`b17143b7`) rather than the v0.9.0 commit when checked just after that release.
- The witness body is `<sha>\n` (`tests/test_docs_publication_contract.py:182`). The existing verifier cache-busts and sends no-cache headers (`tests/test_docs_publication_contract.py:204-206`, `scripts/verify-docs-publication.py:68-92`) because the site's cache can lag about ten minutes (`sdlc/records/1224-...md`).
- The publishers do not all share one dependency: `pypi-publish` needs `[pypi-build, wheel-smoke]` (`release.yml:212-213`), while `homebrew-tap` (`:227`) and `container-publish` (`:285-287`, whose `if: always() && ...` must be extended as well) need only `build`.
- `container-publish` checks out the packaging ref at `release.yml:297-298`; `:305` is the `SOURCE_SHA` API call, not `github.sha`.

## Design

- Write a stable pointer `__biomcp_revision__/latest.txt` beside the revision file in `scripts/copy-markdown-twins.py`, containing the same commit sha. The single-file retention rule then cannot remove it.
- Add a `docs-live` job to `release.yml`. It resolves the tag commit with `gh api repos/$GITHUB_REPOSITORY/commits/$TAG --jq .sha`, fetches the pointer with cache-busting headers and bounded retries that span the deploy and cache window, and passes when the live revision equals the tag commit or is a descendant of it (`gh api repos/$GITHUB_REPOSITORY/compare/<tag-sha>...<live-sha>`). It fails when the live revision is behind or divergent.
- Gate `pypi-publish`, `homebrew-tap`, and `container-publish` on the job: add it to each `needs`, and add `needs.docs-live.result == 'success'` to the container job's `if`. `docs-live` carries no `container_only` gate so it runs on both triggers, which means a container-only backfill also requires the live site to be at or past the tag; the deploy usually catches up, and the alternative would be a gate that silently never passes.
- Extract the compare decision into a `scripts/` helper with a fake-`gh` test, so the pass and fail branches have a mechanical proof instead of living only in a `run:` block.
- Update the provenance pin at `tests/test_release_workflow_provenance.py:57`, which asserts the exact string `needs: [pypi-build, wheel-smoke]`.
- Keep the check out of the five-leg `build` matrix, and keep it read-only with the default token.
- Update `docs/reference/release-process.md:11` and `:82-87` with the gate, the descendant rule, and the cache window.

## Acceptance

1. The helper passes when the live revision equals or is a descendant of the tag and fails when it is behind or divergent, proven by a fake-`gh` test over both branches; a release whose live revision is behind the tag fails before `pypi-publish`, `homebrew-tap`, and `container-publish`.
2. `scripts/copy-markdown-twins.py` writes the pointer, and a contract test pins it without weakening the single-witness assertion.
3. Provenance tests pin the gate's `needs` on all three publishers and the container job's extended `if`.
4. The runbook records the semantics: equal or descendant passes, behind or divergent fails, and the cache window bounds the retries.
5. `make lint`, `make test`, and `make spec` pass on yellow at the pushed SHA, and CI `canonical-gates` is green on main.

## Out of scope

- Changing when the docs site deploys, or making it deploy tags.
- Publishing coordinates changes of any kind.

## Complexity

- Level 3 (contract 1, state and timing 1, reach 1, proof 2, cost of error 2 = 7)
- Reasons: deploy and cache window with bounded retries and an `always()` condition; the fail branch needs a stale live site or a hosted dispatch; a wrong gate strands PyPI, the tap, and the image at once.

## Review

- Design review: REJECT 2026-09-22 (gpt-5.6-sol, medium) — required the stable pointer and gating all three publishers; the split moved this work out of the original 1222 bundle
- Code review: ACCEPT 2026-09-22 (gpt-5.6-sol, medium) — two P2 fixes applied in 73bfc267 (bounded fetch timeouts, default-branch helper checkout)
- Verification: actionlint clean, 43 focused Python tests with mutation checks, fake-`gh` gate test over equal, descendant, behind, divergent, and error cases; see `sdlc/records/1226-gate-a-release-on-the-live-documentation-revision.md`
