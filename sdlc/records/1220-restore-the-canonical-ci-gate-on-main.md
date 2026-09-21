---
base: d4b2758a
head: d5e563e5
---

Restored the canonical CI gate so the release guards actually run.

`canonical-gates` had been failing before any gate ran: the pinned
`bubblewrap=0.9.0-1ubuntu0.1` was gone from the ubuntu-24.04 runner image
(noble-updates now carries `0.9.0-1ubuntu0.3`). The apt installs for
`bubblewrap`, `apparmor`, `apparmor-profiles`, and `ripgrep` are now unpinned
and use `--no-install-recommends`; the AppArmor boundary steps and every
non-apt pin are unchanged. `tests/test_offline_gate_contract.py` now asserts
the exact package token set and that no inline `=` pin is present, instead of
the removed version env vars.

`MAX_PACKAGE_FILES` stays at 1,342. Measured with make-test semantics
(`TMPDIR` inside the worktree) the tree packages 1,342 files at 3111e783 and at
HEAD; the 1,347 reading came from untracked files in the yellow clone. CI
confirms it.

Verified by CI run 35660647512 on main at d5e563e5: all five jobs succeeded,
and `canonical-gates` passed `Canonical lint gate`, `Canonical test gate`, and
`Canonical specification gate`. Those lanes carry
`spec/surface/docker-image.md` and `tests/test_release_workflow_provenance.py`,
so the container-publication guards now run on every push to main.

Code review: ACCEPT 2026-09-21 (gpt-5.6-sol, medium); the two assertion-strength
notes were remediated before the merge.
