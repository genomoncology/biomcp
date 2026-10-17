# 2137 — 0.9 gates on the macOS gate host (build record)

Status at Ian's pause (2026-10-16 evening): branch tip 4fb3dbaea pushed; one
acceptance item (a fully green `make release-gate` run on the M5) is still
open, blocked by contention flakes documented below, not by platform gaps.

## What ran on the M5 (pm job, machine m5, repo biomcp)

Four-gate driver (one pm job running `make lint`, `make test`, `make spec`,
`make release-gate` sequentially, each verdict captured; logs under
`~/workspace/job-runs/biomcp/2137/`):

- 4f2eab4ca (run 1792184801050): lint pass, test fail (26 Python contract
  failures), spec pass, release-gate fail. Diagnosed and fixed in the
  commits after 4f2eab4ca (see Findings).
- c07bf1f00 (run 1792185893715): lint pass, test pass (3 not-network-isolated
  lines in the log), spec pass, release-gate fail at `full-feature-check`:
  clippy 1.95 `--all-targets` findings the routine lint never compiles.
  Fixed at c35126ae1.
- c35126ae1 (run 1792186483379): lint pass, test fail (one rmcp contract
  test, contention), spec pass, release-gate fail (one rmcp contract test,
  contention, different test).
- c35126ae1 (run 1792187230889): lint pass, test pass (3 not-network-isolated
  lines), spec pass, release-gate fail at its internal `make test` on the
  HPO retry-proof (contention flake, see below).

Supporting runs: full Rust suite `--no-fail-fast` 4277/4277 passed at
34769f8cf (run 1792184766591); `cargo clippy --all-targets --all-features`
clean on the host's 1.95 at c35126ae1 (run 1792186437263); Python lane
2 failing (source-package file pin) at a72928d80, clean after c07bf1f00.

## Node

Node v22.22.3 was already installed on the M5
(`/opt/zerobrew/opt/node@22/bin`, first on PATH). main's 0.9 tree carries
no `website/prepare` (a 1.0-line artifact), so no 0.9 gate reads Node;
nothing was installed. Recorded in the ticket.

## Decisions (also in the ticket)

- Gate-host Rust toolchain: the host keeps its zerobrew-managed stable
  (clippy 1.95.0) with no rustup or CI pin installed beside it; the
  repository stays clippy-clean under both the CI pin 1.93.1 and the host's
  newer stable. This branch fixed every 1.95-only finding found so far:
  shell.rs `unnecessary_fold`, opencitations.rs `while_let_loop` (507f90d09),
  and the three `--all-targets` findings at c35126ae1 (rmcp fixture reader
  `while_let_loop`, ranking tests `explicit_into_iter_loop`, ca_bundle
  macOS-conditional `unused_mut`/`useless_vec`).
- Gate-host Python: `.python-version` pins uv to the 3.12 line CI already
  pins, so the lane venv stops following the host's newest Python (3.14
  tripped the `release/inspect.py` stdlib shadow through argparse's
  colorize import chain).

## Findings

- `TMPDIR`: macOS `TMPDIR` (`/var/folders/...`) holds the `/var` symlink,
  which the GenCC store's `O_NOFOLLOW` secure chain rejects; the Makefile's
  exported worktree TMPDIR is symlink-free, so gates are unaffected. Run
  single Rust tests through make or with TMPDIR pinned at the worktree.
- `CARGO_INCREMENTAL=1` plus the host's sccache wrapper fails compiles
  outright; the gates run with the host default (incremental off).
- Product bug found and fixed: `install.sh` lowercased the checksum record
  with Bash 4 expansion, which the bundled Bash 3.2 rejects — every stock
  macOS install died at checksum verification (878234624, with docs copy).
  `tools/biomcp-verify-live` had the same Bash-4-ism (ec9f929ec).
- The cache-clear tests steered the CLI with `XDG_CACHE_HOME`, which the
  macOS platform root ignores: the spawned CLI cleared the host user's real
  `~/Library/Caches/biomcp` http tree instead of the fixture (regenerable
  cache; the fix, 62da8b529, routes the fixture root through
  `BIOMCP_CACHE_DIR`).
- The credential-redaction fixture's 127.0.0.2 bind is macOS-invalid;
  fixture socket windows widened to ten seconds and read-phase connection
  churn no longer kills the fixture thread (62da8b529, 88753d8e8, a839cc5cc).
- The HPO retry proof flaked under load twice (3 vs 4 attempts) because
  the client's connection pool can hand an attempt a dying connection;
  rewritten to count served requests on kept-alive connections (34769f8cf).
  It still flaked once more under release-gate rerun at heavy host load —
  the remaining open item below.

## External blocker

`cargo deny check advisories` fails on the zip 0.6.6 symlink advisory —
ticket 2143 owns the fix. On the M5 `make lint` passes (the host's cached
advisory DB predates the entry); CI's lint lane fails on it. Landing needs
the post-2143 rebase, then CI green at the exact tip.

## Remaining at the pause

1. One clean four-gate run at the branch tip: `make lint`, `make test`,
   `make spec` each passed green on the M5 in run 1792187230889;
   `make release-gate` still needs a run without a contention flake. The
   two flake families seen only above ~20/16 host load: the HPO retry proof
   (once after the rewrite) and individual `rmcp_client_contract` tests
   (two different tests, once each). Both pass isolated and in full suites
   at moderate load (4277/4277 twice).
2. CI green at the tip: 4fb3dbaea fixes an E0597 the Linux-only build
   caught in the ca_bundle gating (macOS cfg's the block out, so the M5
   compiled clean); its CI run was in flight at the pause. The only other
   expected failure anywhere on the branch is the zip advisory (2143).
3. Landing (coordinator): rebase after 2143 lands, ci.yml green at the
   exact post-rebase tip, then land.

## Machine notes

Nothing ran on Yellow or the Beelink (the Beelink only ran python-only
contract checks and read-only git/ssh). No pm jobs are running for 2137 at
the pause. /tmp scratch on the M5 was removed; pm job-run logs under
`~/workspace/job-runs/biomcp/2137/` keep the full evidence.
