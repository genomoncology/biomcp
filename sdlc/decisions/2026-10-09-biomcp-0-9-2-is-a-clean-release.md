# BioMCP 0.9.2 is a clean release, and commits trigger nothing

Copied from the shared decisions on 2026-10-20; this repo's copy is now the home of this ruling.

Decided 2026-10-09. Ian ruled on the reviewer's five recommendations after the review that filed BioMCP tickets 2042 to 2044.

- Every finding in 2042, 2043 and 2044 blocks the 0.9.2 tag, including the slowdowns, refusal text, Darzalex, test gaps and size-record cleanup. Ian: "I want a clean release. I don't really want tickets laying around." No 0.9.2 finding moves to a later milestone without Ian.
- Tickets 2039 and 2040, which the team deferred to 0.9.3, move back into 0.9.2 under the same rule.
- Nothing runs automatically on commit. The local `.git/hooks/pre-commit` in Ian's BioMCP checkout is removed, and BioMCP ticket 2005 is withdrawn. Agents do not run `scripts/install-pre-commit-hook`. Checks run on purpose ~~on Yellow before each push and again in CI~~. Amended 2026-10-17: checks run through `pm job` on the build machine, and GitHub Actions run only for public releases, as `2026-10-17-github-actions-only-for-public-releases.md` says.
- No GitHub secret and no forbidden-names check. Ian ruled on 2026-10-10 that leaving other projects' names out of public repos is a plain writing habit and needs no guard. The team removes the check.
- Ian authorizes the 0.9.2 release rehearsal in the public scratch repository `genomoncology/biomcp-release-rehearsal` on the release candidate.
- The WHO local data path shows only in the terminal, never in JSON or MCP output.

Ian can overturn any of these.
