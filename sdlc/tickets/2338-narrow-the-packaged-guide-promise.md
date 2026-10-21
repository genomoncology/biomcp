# 2338 — Promise only what the packages ship

Status: OPEN.

Milestone: 0.9.4

## Outcome

BioMCP's published text promises only what its release packages actually ship. The executable embeds the agent skills, through `biomcp skill`, `biomcp skill render` and `biomcp skill install`, and the documentation lives at https://biomcp.org/. AGENTS.md, the README, the install docs and the packaged agent guide stop promising `share/biomcp/docs/` or `share/biomcp/skills/` and point to those commands and the site instead. Delete the unused release/package.py guide-copy path and its tests.

- Proof: a repository check fails when published text names a `share/biomcp/` path that no release artifact contains; `biomcp skill render` and `biomcp skill install` work from the installed wheel in the existing installed-wheel smoke; release/package.py is gone and no build, test or workflow refers to it.

## Evidence

- On 2026-10-21, a fresh independent release review of integration 3c8aee87a found that AGENTS.md promises installed `share/biomcp/docs` and `share/biomcp/skills`. The release tar and zip, the maturin wheels, install.sh and Homebrew ship only the executable. An unused release/package.py has a tested guide-copy path.
- The binary already embeds the skills (src/cli/skill/), so shipping copies would duplicate them. The PM chose to narrow the promise instead of building and testing a second asset path.
