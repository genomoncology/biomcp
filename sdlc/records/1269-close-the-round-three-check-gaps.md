---
ticket: 1269
head: 346068be
merge: 535747d9
---

# 1269 — the round-three check gaps

## What changed

Every release job's step list is pinned exactly (pypi-publish included). Expression-form `BASH_ENV` is banned at every nesting level with state and kind allowlists. The review-status grammar became a whole-line allowlist: a status declaration inside any decoration with a state word fails, and REJECT needs a later ACCEPT or an inline resolution word. The wait ratchet resolves aliases, counts stored clocks, and requires every ceiling or pin raise to cite a ticket whose review carries ACCEPT.

## Evidence, read from the API on 2026-09-29

- Branch head `346068be`; merge `535747d9`.
- The merge commit's readable CI runs (36530006688, 36530006673, both push to main 2026-09-29) failed in the `full-features` job at `make full-feature-check`; the job logs have expired, and the failures were chased in the 1271-1274 flake round whose final state is green on main. No green run is claimed for this merge.
- Code review: the ticket file's verdict lines are the record.
