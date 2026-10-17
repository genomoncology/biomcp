---
flow: build
priority: 1
deps: []
---

# 2137: Run the 0.9 gates on the macOS gate host

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 99f2e12, accept

## Outcome

The macOS gate host runs every 0.9 build, test, lint, gate and release check on `main` through pm job. The Linux gate host runs only an occasional Linux check. No check, script or document on `main` names the Linux gate host as the place 0.9 builds.

## Evidence

- `make lint` cannot start on macOS: `tools/bootstrap-lint-tools` ships no Darwin-arm64 archive.
- `make spec` is Linux-only: `scripts/run-specs.sh` and `scripts/fixture-supervisor.sh` read `/proc` unconditionally, and the fixture scripts need Bash 4.2 or later while macOS ships Bash 3.2.
- `make test` refuses on macOS: `tools/run-offline` refuses any platform without its Linux network sandbox.
- `website/prepare` pins Node v22.22.3, which the macOS gate host must install beside its current Node.
- Ticket 2098 fixed these on the paused 1.0 line in four commits: b1c66a853 (lint tools on Darwin-arm64, run-offline pass-through on macOS), f5a2c687d (portable fixture process data), 7bad55dfb (build version through cargo metadata in `tools/with-build-identity`), and 6f8270904 (empty spec worker arrays on Bash 3.2). 2098's rulings carry over: the Node pin stays, and `tools/run-offline` on macOS runs the command without the sandbox and prints one stderr line saying the run is not network-isolated. Linux CI stays the proof that the suite runs offline.

## Decisions

- Gate-host toolchain (the 2143 drift finding): the macOS gate host keeps its user toolchain, the zerobrew-managed stable Rust (clippy 1.95.0 today), and installs no rustup or CI-pinned 1.93.1 beside it. The repository stays clippy-clean under both the CI pin and the host's newer stable (the redaction and DOI fixes in this branch), so the gate host surfaces new lints before the pin moves and CI keeps proving the pin. Duplicating the pinned toolchain on the host would redo CI's job and hide the drift.
- Node v22.22.3 is already installed on the macOS gate host (`/opt/zerobrew/opt/node@22/bin` on PATH), and main's 0.9 tree carries no `website/prepare`, so no gate reads Node: nothing was installed.

- Gate-host Python: the repository pins `uv` to the 3.12 line through `.python-version`, matching the `PYTHON_VERSION` CI already pins, so the gate host's `uv sync` builds the lane venv on the interpreter the release scripts support instead of the host's newest Python; the host's 3.14 exposed the `release/inspect.py` stdlib shadow through argparse's colorize import chain.

## Findings

- Single Rust tests on the gate host run through make, or with `TMPDIR` pointed at the worktree: macOS `TMPDIR` (`/var/folders/...`) holds the `/var` symlink, which the GenCC store's `O_NOFOLLOW` secure chain rejects, while make exports the symlink-free `.cache/routine-test-tmp` for the test goals.
- `CARGO_INCREMENTAL=1` breaks compiles on the gate host because the host's sccache wrapper cannot cache incremental crates; the gates run with the host default (incremental off).

## Work

1. Port the four 2098 commits onto `main`, adapting them to `main`'s scripts. Take nothing else from the 1.0 line.
2. Install Node v22.22.3 in the macOS gate host's user toolchain directory if it is missing, so `website/prepare` finds it.
3. Replace any wording on `main` that names the Linux gate host as the 0.9 build host, in checks, scripts and the release checklist.

## Acceptance

1. In one pm job on the macOS gate host at the ticket branch tip, `make lint`, `make test`, `make spec` and `make release-gate` each run to their own verdict, and no step exits on a missing platform.
2. The `make test` log carries the not-network-isolated line.
3. ci.yml is green on the landing tip, so Linux behavior is unchanged.
4. The changelog carries a bullet for this ticket.
