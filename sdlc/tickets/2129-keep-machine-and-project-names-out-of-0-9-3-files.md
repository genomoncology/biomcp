# 2129 — Keep machine and project names out of 0.9.3 files

Status: OPEN.

Milestone: 0.9.3

## Outcome

Nothing added or changed on main since v0.9.2 names a build machine or another project. Shipped tools and test data say "the Linux gate host" or name no host at all.

## Evidence

Found by the 0.9.3 adversarial review on 2026-10-16: since v0.9.2, the build-machine names appear about 67 times, in the 2069 record and in tickets 2089, 2098, 2106, 2107 and others. `tools/measure-0-9-3-baseline` hard-codes a machine name and its gate lock file, and `testdata/sources/capture-receipts.json` names another project. The repository is public and talks only about itself.

- Decisions:
  - Replace machine names with their role ("the Linux gate host", "the macOS gate host"); remove other project names or replace them with what the reader needs to know.
  - The measure script takes its host lock path from an environment variable with no machine-specific default.
  - Files older than v0.9.2 stay as they are.
- Changes: claims `tools/measure-0-9-3-baseline` `testdata/sources/capture-receipts.json` and the `sdlc/` files changed since v0.9.2 that carry the names.
- Proof: `git diff v0.9.2 HEAD` shows no build-machine or other-project name in added lines; the measure script's tests pass.
- Order: safe alongside anything; touch `tools/measure-0-9-3-baseline` after 2123 lands.
