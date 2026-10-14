# 2064 — Fix the install paths the 0.9.2 test found

Status: OPEN.

Milestone: 0.9.3

Reviews: revision c139e21, accept

Reviews: revision 88b2d9a, accept

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

## Amendment 2026-10-13

Ian ruled mid-build: drop the `biomcp-cli` wrapper instead of fixing it. Delete `src/main_biomcp_cli.rs`, its `[[bin]]` entry in Cargo.toml, its packaging references, and its tests; the PyPI package keeps its name (`biomcp-cli`) and installs only the `biomcp` binary. The changelog bullet and the install docs tell uvx users to run `uvx --from biomcp-cli biomcp serve` (uvx needs `--from` because the package name and the command name now differ). The "wrapper resolves a symlinked sibling" change and its proof case are dropped entirely. Every other item above stands: the install.sh version read and receipt, the Python 3.10 floor, Homebrew macOS-only docs, Docker pin guidance, the maintainer-note drop, the formula `test do` block, and the release-notes-from-changelog step.

Built 2026-10-13 on `ticket/2064-install-paths`.

- Red probes on main (`probe/2064-red-tests`, deleted after recording): nine Python tests failed as filed — the pinned install refused a staged release whose `version` banner ends with the build field, the latest install wrote that field into the receipt, the docs lacked the Python 3.10 floor and the Homebrew/Docker statements, the rendered formula carried no `test do` block, and `create-draft` created a draft with no notes. The Rust probe failed with exit 126, "sibling is not a regular file", through a symlinked sibling.
- After the fix: `tests/test_public_installer_checksum.py` runs the installer against a staged release with `--version 0.9.2` (both banner shapes) and reads back `version: 0.9.2` from the receipt, and still refuses a mismatched pin; `tests/test_release_workflow_provenance.py` renders the formula and finds the `test do` block; `tests/test_release_draft_notes.py` executes the render step's script and proves the draft carries the exact changelog section. The symlinked-sibling proof named here was dropped by the amendment below.
- Live pass (Yellow, built binaries at `18c64353`): `biomcp --version` prints `biomcp 0.9.2+g18c64353` and `biomcp version` ends with the build field, confirming the second-word read; the wrapper through a symlinked sibling prints the same banner.
- The 0.9.3 changelog section and its expected-ticket set carry the 2064 bullet.

Built 2026-10-13 under the amendment, final head `77ac562437ce067e1949930fda736684fe0fd2fe` on `ticket/2064-install-paths`.

- Dropped: `src/main_biomcp_cli.rs`, its `[[bin]]` entry, `tests/biomcp_cli_shim.rs`, `tests/test_biomcp_cli_shim_contract.py`, the shim in `release/package.py`/`build_target.py`/`inspect.py` with their tests, the `biomcp-cli` member in the wheel floor scanner, the formula's `biomcp-cli` symlink with its spec line, and the shim entry in the test-wait inventory. The source-package file cap moved 1418 → 1416.
- Gates on the final state (Yellow): `make lint` PASS (fmt, clippy, ruff, shellcheck, both ratchets); `make test` PASS (4202 Rust tests, Python contracts, strict mkdocs; one load-induced flake in `rmcp_client_contract` on the first run passed standalone and on the rerun); `make spec` PASS (7 pages, after the spec page dropped the removed formula line).
- Live pass (Yellow, `77ac56243`): `cargo build --bins` produces no `biomcp-cli` binary; `biomcp --version` prints `biomcp 0.9.2+g77ac5624` and `biomcp version` ends with the build field; a maturin-built wheel's only script member is `data/scripts/biomcp` and no `biomcp-cli` member exists anywhere in it.
- CI: intermediate heads `78204aa92` (fmt nit in the since-deleted shim test), `db72da173` (package file cap), and `bd27ba707` (spec page line) failed as described; the final head `77ac56243` finished green (run `38483210845`).
- The install docs now tell uvx users `uvx --from biomcp-cli biomcp serve`, and the 0.9.3 bullet says the same.
