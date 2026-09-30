# 1279 — the second review round

Filed 2026-09-30 (retroactively, as its own ticket): the 2026-09-29 review of the CI rework and records round (`sdlc/issues/2026-09-29-review-of-the-ci-rework-and-records-round.md`) drove this work, and the round's record existed before this ticket file did. The 2026-09-30 go-request review flagged the missing file.

## Scope

- The classify script diffs the merge-base with origin/main under a directory allow list that excludes executable markdown (`spec/`, `skills/`, `src/`, the compiled CLI reference); the behavioral suite `tests/test_ci_classify_push_script.py` drives the real script against scratch repos.
- The GWAS no-store pin reads `RequestBuilder::extensions()` on the middleware builder (reqwest-middleware 0.4.2 exposes it publicly); the first attempt read reqwest's private `Request::extensions()` after `build()` and failed with E0624 in CI run 36619750222.
- `NoTrustFailureStrategy::handle` and the plain-send retry loop are driven by real tests through `reqwest_middleware::Error::Middleware`; the plain-send path (Enrichr, UniProt) stops at the first trust failure.
- The changelog gate's under-2000 cap was removed in favor of date-shape rejection; `--release --locked` is asserted per command; the raise-review check rejects promise-shaped ACCEPT phrases; the README and hero checks are strict; the licensing page headings match the registry with a heading-aware tier test.
- The records round: real verdicts and green runs where they exist, dispositions completed, shorthand expanded, the named files unwrapped.

## Evidence

- Branch `tickets/1279-review-round`, final head `5e3426c2`, green branch run 36684435506 (all eight jobs); main green at 36687060448 (5e3426c2) and 36693388098 (7d2c6c13 after the changelog coverage fix, branch run 36689893012).
- Twelve intermediate commits on the branch failed their runs (compile and pin cycles through 36681892037); each failure was fixed on the branch before the green tip. The go-request review recorded the bisection cost honestly.

## Verdicts

- Code review: ACCEPT 2026-09-29 by a fresh read-only reviewer on the Claude vendor (github-copilot/claude-sonnet-5, dispatch 4e671b58-6864-4742-83cc-7a3c3163282e), covering the five commits after ticket 1268's ACCEPT and the round's fixes; three findings folded (the classify behavioral suite, dead code after a continue, stale size-inventory numbers). The reviewer verified by reading files at `767c7292` and could not read commit diffs; the commits after its ACCEPT (`8817ba85` through `7d2c6c13`) carry the changelog, the version bump, the record pins and the compile fixes, and the 2026-09-30 go-request review re-verified their landed state with mutations.
