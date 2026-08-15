---
flow: build
priority: 10
---

# 0988: Restore trustworthy hosted CI

## Outcome

The exact BioMCP commit can complete its canonical Linux and Windows hosted
checks, every GitHub Action invocation is pinned to a full immutable commit,
and the three known fixed dependency alerts are cleared.

## Current facts

GitHub Actions run `31815254414` for commit
`349abee92de09d6740aa4faa6651360bb494d8bf` did not provide release evidence.
The canonical Linux job could not load the shortened `astral-sh/setup-uv`
revision, and the Windows job could not compile the unstable and incorrectly
typed `MetadataExt::number_of_links()` call in `src/cache/private.rs`.

The current action-pin test only recognizes lines beginning exactly with
`uses:` and misses normal YAML list entries beginning with `- uses:`. The
release candidate gate also invokes the canonical gates without installing all
of their pinned tools. Dependabot reports fixed releases for `quinn-proto` and
the development-only `pymdown-extensions` documentation dependency.

After those changes landed, exact-head run `31888380333` proved Windows,
full-feature, generated-source, and repository checks green but exposed one
remaining Ubuntu 24.04 boundary failure. The runner's Bubblewrap reaches its
new user/network namespaces, then AppArmor denies the netlink operation that
assigns the loopback address (`RTM_NEWADDR`). The verifier and test command
never start. This machine succeeds because it has Ubuntu's scoped
`bwrap-userns-restrict` profile installed and enforced. `tools/run-offline`
then obscures the bootstrap failure with a secondary missing-sentinel message.

## Scope

- Pin every action invocation under `.github/workflows/` to a full 40-character
  commit and make the repository test inspect the actual YAML syntax.
- On Windows, obtain the opened file's link count through a stable operating
  system API and reject hard-linked managed files without spawning `fsutil`.
- Give the release candidate gate the same pinned lint, test, specification,
  and sandbox tools required by the canonical gates.
- Update `quinn-proto` from `0.11.14` to at least `0.11.15` to clear
  GHSA-4w2j-m93h-cj5j. Update the development-only `pymdown-extensions` from
  `10.21.3` to at least `11.0.1` to clear GHSA-gm37-52c6-37mw and
  GHSA-9xwg-3r6f-jcx2. Do not perform unrelated dependency upgrades.
- In both the canonical CI job and the private candidate gate, install pinned
  `apparmor` and `apparmor-profiles`
  `4.0.1really4.0.1-0ubuntu0.24.04.7` beside pinned Bubblewrap. Copy the
  distro-owned `bwrap-userns-restrict` profile from its extra-profiles path,
  load it with `apparmor_parser -r`, assert
  `kernel.apparmor_restrict_unprivileged_userns` remains `1`, and run
  `tools/run-offline -- true` before compilation. Do not disable the global
  restriction, make Bubblewrap setuid, share the host network, or run the test
  command with elevated privileges.
- Make `tools/run-offline` distinguish a sandbox bootstrap failure from an
  ownership-mapping failure. Create the ownership sentinel as soon as the
  verifier starts, report isolation success only after the verifier has run,
  preserve Bubblewrap's original error and exact child status, and retain the
  existing privilege, public-network denial, loopback, Unix-socket, ownership,
  cleanup, and marker-revalidation proofs.

The Windows implementation must keep the existing open-handle safety property:
it must inspect the file that BioMCP opened, not reopen or identify it only by
path, and fail closed if handle metadata cannot be read. Remove all `fsutil`
link-count subprocesses from this module. Focused unit tests in
`src/cache/private.rs` may directly exercise `open_managed_read` with a
hard-linked file. Existing tests in `tests/managed_state_permissions.rs` may be
extended to prove the managed-tree path uses the same stable check. Existing
action and release-workflow assertions in
`tests/test_release_stage_workflow.py` may be restated or extended.

## Acceptance

- A focused test fails for a shortened or symbolic action revision in any
  workflow, including `- uses:` entries, and passes for full commit pins.
- A Windows-focused test calls `open_managed_read` itself and rejects a
  hard-linked file by inspecting its opened handle. The managed-tree path also
  rejects hard links without `fsutil`, and metadata failures reject access.
- Release-workflow tests prove the candidate gate installs every tool its
  canonical commands require.
- Workflow tests prove both authoritative Ubuntu jobs install and load the
  exact scoped AppArmor profile, retain the unprivileged-user-namespace
  restriction, and run the offline preflight before expensive compilation. A
  deterministic failed-Bubblewrap test reports that the verifier never started
  without falsely reporting an ownership mismatch; successful and child-exit
  paths retain their current proofs.
- Lockfiles contain the fixed dependency versions; GHSA-4w2j-m93h-cj5j,
  GHSA-gm37-52c6-37mw, and GHSA-9xwg-3r6f-jcx2 are absent from the resulting
  dependency alerts; and the focused dependency, workflow, and managed-state
  tests pass.
- `make lint` and `git diff --check` pass locally; the exact landed commit's
  hosted canonical, full-feature, generated-source, repository, and Windows
  jobs are green before this ticket is considered release evidence.

## Dependencies

None.

## Review

- Design review: accepted after clarifying direct open-handle coverage,
  fail-closed behavior, exact advisories, and the complete outcome; reopened
  AppArmor design accepted with the global restriction retained and the child
  privilege/network proofs unchanged
- Code review: accepted after strengthening the Windows managed-tree proof to
  require ordinary success followed by the exact hard-link rejection; reopened
  after hosted Ubuntu exposed the missing scoped AppArmor setup
