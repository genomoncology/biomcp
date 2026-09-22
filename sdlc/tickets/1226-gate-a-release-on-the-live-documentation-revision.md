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
- The witness body is `<sha>\n` (`tests/test_docs_publication_contract.py:182`). The existing verifier cache-busts and sends no-cache headers (`:218-227`) because the site's cache can lag about ten minutes (`sdlc/records/1224-...md`).
- The publishers need only `build`: `pypi-publish` (`release.yml:118-119`), `homebrew-tap` (`:227`), and `container-publish` (`:285-287`, whose `if: always() && ...` must be extended as well).
- `container-publish` resolves the revision with `github.sha` (`release.yml:305`), which is main's tip on a dispatch, not the tag commit.

## Design

- Write a stable pointer `__biomcp_revision__/latest.txt` beside the revision file in `scripts/copy-markdown-twins.py`, containing the same commit sha. The single-file retention rule then cannot remove it.
- Add a `docs-live` job to `release.yml`. It resolves the tag commit with `gh api repos/$GITHUB_REPOSITORY/commits/$TAG --jq .sha`, fetches the pointer with cache-busting headers and bounded retries that span the deploy and cache window, and passes when the live revision equals the tag commit or is a descendant of it (`gh api repos/$GITHUB_REPOSITORY/compare/<tag-sha>...<live-sha>`). It fails when the live revision is behind or divergent.
- Gate `pypi-publish`, `homebrew-tap`, and `container-publish` on the job: add it to each `needs`, and add `needs.docs-live.result == 'success'` to the container job's `if`.
- Keep the check out of the five-leg `build` matrix, and keep it read-only with the default token.
- Update `docs/reference/release-process.md:11` and `:82-87` with the gate, the descendant rule, and the cache window.

## Acceptance

1. A release whose live revision is behind the tag fails before `pypi-publish`, `homebrew-tap`, and `container-publish`; a live revision equal to or newer than the tag passes.
2. `scripts/copy-markdown-twins.py` writes the pointer, and a contract test pins it without weakening the single-witness assertion.
3. Provenance tests pin the gate's `needs` on all three publishers and the container job's extended `if`.
4. The runbook records the semantics: equal or descendant passes, behind or divergent fails, and the cache window bounds the retries.
5. `make lint`, `make test`, and `make spec` pass on yellow at the pushed SHA, and CI `canonical-gates` is green on main.

## Out of scope

- Changing when the docs site deploys, or making it deploy tags.
- Publishing coordinates changes of any kind.

## Review

- Design review: pending
- Code review: pending
