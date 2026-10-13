# 2064 — Fix the install paths the 0.9.2 test found

Status: OPEN.

Milestone: 0.9.3

## Outcome

Every documented install path works as the docs say: a pinned install script run, the `biomcp-cli` wrapper on every platform, and install docs that state the Python floor, which platforms Homebrew covers, and how to get a current Docker image.

## Evidence

Filed 2026-10-13 from the post-release install and product test of v0.9.2 (tag `4049666f5`), run on Linux x86_64 and macOS arm64 against every install channel.

- `curl -fsSL https://biomcp.org/install.sh | bash -s -- --version 0.9.2` fails on both platforms with "requested 0.9.2 but staged binary reported 2026-10-13T00:14:34-04:00)". `install.sh:350` takes the last word of `biomcp version`, which is now the build-time field. A latest install succeeds but writes that build time as the receipt's version. 0.9.1 fails the same way.
- After `uv tool install biomcp-cli==0.9.2` on macOS, `biomcp-cli --version` exits 126 with "sibling is not a regular file", because uv's bin folder holds symlinks. It works on Linux and in a pip venv.
- The wheels need Python 3.10 or later. A venv from macOS's system Python 3.9 finds no matching distribution, and the install docs do not say so.
- The Homebrew formula has only an `on_macos` section and no `test do` block. The docs neither say Homebrew is macOS-only nor drop the maintainer note that the tap "must exist… one-time release prerequisite".
- `docker run ghcr.io/genomoncology/biomcp` runs whatever `latest` image a machine already holds; on one test machine that was 0.9.0.
- The v0.9.2 and v0.9.1 GitHub releases carry no release notes; v0.9.0 does.

- Starts from: `install.sh` and `docs/install.sh` (CI keeps them identical), `src/main_biomcp_cli.rs`, the install docs, the release workflow's release-notes step.
- Keeps: the receipt format, `update --check` and `uninstall`.
- Changes: read the version from `biomcp --version` (or the second word of `biomcp version`) and write the true version into the receipt; let the wrapper resolve a symlinked sibling; state the Python 3.10 floor, that Homebrew is macOS-only, and to pull or pin the Docker tag; drop the maintainer note; give the formula a `test do` block in the release workflow's formula render; publish the changelog section as the GitHub release notes.
- Proof: an outside-in test runs the install script with `--version` against a staged release and passes, and reads back the receipt's version; a test runs the wrapper through a symlinked sibling; the rendered formula carries a test block; a release rehearsal's draft release carries the 0.9.3 notes.
- Defers: a Linux Homebrew formula.

Built 2026-10-13 on `ticket/2064-install-paths`.

- Red probes on main (`probe/2064-red-tests`, deleted after recording): nine Python tests failed as filed — the pinned install refused a staged release whose `version` banner ends with the build field, the latest install wrote that field into the receipt, the docs lacked the Python 3.10 floor and the Homebrew/Docker statements, the rendered formula carried no `test do` block, and `create-draft` created a draft with no notes. The Rust probe failed with exit 126, "sibling is not a regular file", through a symlinked sibling.
- After the fix: `tests/test_public_installer_checksum.py` runs the installer against a staged release with `--version 0.9.2` (both banner shapes) and reads back `version: 0.9.2` from the receipt, and still refuses a mismatched pin; `tests/biomcp_cli_shim.rs::shim_resolves_a_symlinked_sibling` forwards through a symlinked sibling with identical output and exit status; `tests/test_release_workflow_provenance.py` renders the formula and finds the `test do` block; `tests/test_release_draft_notes.py` executes the render step's script and proves the draft carries the exact changelog section. All green locally and on Yellow.
- Live pass (Yellow, built binaries at `18c64353`): `biomcp --version` prints `biomcp 0.9.2+g18c64353` and `biomcp version` ends with the build field, confirming the second-word read; the wrapper through a symlinked sibling prints the same banner.
- The 0.9.3 changelog section and its expected-ticket set carry the 2064 bullet.
