# 2338 — Promise only what the packages ship

Status: OPEN.

Milestone: 0.9.4

## Outcome

BioMCP's published text promises only what its release packages actually ship. The executable embeds the agent skills, through `biomcp skill`, `biomcp skill render` and `biomcp skill install`, and the documentation lives at https://biomcp.org/. AGENTS.md, the README, the install docs and the packaged agent guide stop promising `share/biomcp/docs/` or `share/biomcp/skills/` and point to those commands and the site instead. Delete the unused release/package.py guide-copy path and its tests.

- Changes: `AGENTS.md`, `README.md`, `docs/getting-started/installation.md`, `tests/test_public_skill_docs_contract.py`, `release/package.py`, `release/build_target.py`, `release/inspect.py`, `tests/test_release_package.py`, `tests/test_release_targets.py`, `tests/test_release_smoke_script.py`, `tests/feature-links.json`, `CHANGELOG.md`. Narrow the guide promise, add the published-text contract, delete the unused package-builder/inspector dependency chain and its private tests, and preserve the identity regression for the production release smoke command. The existing installed-wheel smoke owns installed skill proof.

- Proof: a repository check fails when published text names a `share/biomcp/` path that no release artifact contains; `biomcp skill render` and `biomcp skill install` work from the installed wheel in the existing installed-wheel smoke; release/package.py is gone and no build, test or workflow refers to it.

## Evidence

- On 2026-10-21, a fresh independent release review of integration 3c8aee87a found that AGENTS.md promises installed `share/biomcp/docs` and `share/biomcp/skills`. The release tar and zip, the maturin wheels, install.sh and Homebrew ship only the executable. An unused release/package.py has a tested guide-copy path.
- The binary already embeds the skills (src/cli/skill/), so shipping copies would duplicate them. The PM chose to narrow the promise instead of building and testing a second asset path.

- Developer outcome before code: remove promises of separately installed guide files; users read or install the embedded skills through the CLI and read documentation on the website. Protect the published text with one contract check. Starting source: `9ce416c29d4637e6ee3b80c8a707b941a62e6f63`; ticket authorization from `bfcbfc2f5876a6a3dc47a8c4f657266b0075bed6`. The outside-in contract scans the published guide surface for standalone `share/biomcp/` references while allowing real platform application-data directories.

- Current-main reproduction: Gator job `j2697` on `9ce416c29d4637e6ee3b80c8a707b941a62e6f63` ran `python3 -c 'from pathlib import Path; import re; paths=[Path("AGENTS.md"),Path("README.md"),*Path("docs").rglob("*.md"),*Path("skills").rglob("*.md")]; bad=[str(p) for p in paths if re.search(r"(?<![\w/])share/biomcp/",p.read_text())]; assert not bad, f"Published guides promise unshipped paths: {bad}"'`. Exit 1: `AssertionError: Published guides promise unshipped paths: ['AGENTS.md']`. The permanent contract uses the same published surface and also rejects absolute asset paths; it permits real `.local/share/biomcp/` application-data paths.

- Dependency inventory: repository-wide references outside SDLC to `release/package.py`, `release/build_target.py` and `release/inspect.py` occur only within that unused chain and its two test modules. Current workflows build native archives directly and wheels through maturin. No workflow, script, Make target or tool calls the deleted chain. Preserve `test_installed_command_smoke_requires_exact_structured_identity` in `tests/test_release_smoke_script.py` because it exercises the production `release/smoke.py` command. Existing production glibc, candidate and signing tests remain.

- Affected checks: Gator `j2726` applied source patch SHA-256 `056f71abb176638b4f8c2b6a4cb1e0d11fbf048be2ddd8d037c4ca84bb65675d` to published authorization base `bfcbfc2f5876a6a3dc47a8c4f657266b0075bed6`. The published-skill docs, installation docs and affected smoke contracts passed (18 passed, 2 unrelated binary-build cases deselected); Ruff passed for both changed test modules. `j2714` was cancelled while unplaced, and `j2725` failed before tests because a ticket-record patch was applied to main rather than its authorization branch; neither supplies test evidence.
