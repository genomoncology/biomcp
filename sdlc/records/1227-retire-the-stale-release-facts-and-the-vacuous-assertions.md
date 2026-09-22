---
base: 3b0c8fcf
head: e8094a1d
---

Retired the stale release facts and the assertions that could not fail.

`architecture/technical/overview.md` now describes the single release workflow
instead of the retired two-step stage/promote process, and a new guard rejects
four recorded retired phrases. The `... or True` and `assert True` lines in
`tests/test_docs_changelog_refresh.py` are gone, and the surviving assertions
were mutation-checked against deliberately wrong sentences. `release/container.py`
now derives its recorded base from `Dockerfile:1`'s `ARG RUNTIME_IMAGE` instead
of the retired bookworm digest, with a test that runs the record end to end and
also asserts the Dockerfile actually consumes the argument. The sentence about
the retired `release/` package now says it stays on disk but is not the release
path. The version facts at `overview.md:268` were inspected and left alone:
`scripts/check-version-sync.sh` exits 0 and pins the development-candidate
mapping and the latest stable tag exactly as the sentence claims.

Evidence: 46 focused Python tests pass; mutation checks fail each strengthened
assertion; `make lint` and `make test` on yellow at e8094a1d pass (3756 Rust
tests, 966 Python tests, 3 skipped).

Reviews: the design review rejected the first draft, which mixed five
subsystems; the split moved the boundary item to ticket 1220's settled decision
and the cache flake to ticket 1228. The code review accepted with three notes;
the guard was made less order-sensitive, the container test gained the
`FROM ${RUNTIME_IMAGE}` assertion, and the stale test name was left in place
because `sdlc/issues/2026-09-18-release-workflow-tests-describe-a-retired-workflow.md`
still cites it.
