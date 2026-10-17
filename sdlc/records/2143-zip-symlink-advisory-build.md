# 2143 build record — zip 9.0.4 past the symlink advisory

## Change

- `Cargo.toml`: zip `0.6` → `9`, still `default-features = false`. The 0.6
  feature `deflate` no longer means the plain flate2 codec: in zip 9 it pulls
  zopfli and the zlib-rs backend. `deflate-flate2` is the faithful
  translation — the deflate codec through flate2's default rust backend and
  nothing else — and zip 9's read path gates Deflated decompression on
  `deflate-flate2` itself, so nothing smaller works.
- `src/cli/update.rs`: `ZipFile::name()` now returns
  `ZipResult<Cow<'_, str>>`; the name error maps to the same
  `BioMcpError::Api` "ZIP error" shape the surrounding by-index and
  open errors use. The rsplit/unwrap_or matching logic is unchanged.
- `src/cli/update/tests.rs` and `src/sources/europepmc/tests/parsing.rs`:
  `FileOptions::default()` became non-`Copy` (`ExtendedFileOptions`); both
  test helpers use `SimpleFileOptions::default()`, the `Copy` alias with the
  same defaults (Deflated stays the default compression method). No test
  assertion changed.
- `src/sources/europepmc.rs` needed no edit: `ZipArchive::new`, `len`,
  `by_index`, `name_raw`, `is_dir`, `size` are unchanged in zip 9.
- `Cargo.lock`: zip 0.6.6 → 9.0.4, adds typed-path 0.12.3, drops byteorder
  1.5.0 (zip 0.6 only), moves libc 0.2.182→0.2.190 and nix 0.31.2→0.31.3
  (zip 9's requirements), reuses the locked indexmap/memchr/crc32fast.
  30 lines of diff; everything else unchanged.
- `CHANGELOG.md`: one Internal bullet citing 2143.

## Lockfile method

`cargo generate-lockfile` through a gate job produced a host-target-only
lock: it dropped every `windows_*`, `redox_syscall`, wasm/wit entry (36
packages), which would have broken the Windows lane at landing. Instead the
lock came from `cargo update -p zip --precise 9.0.4` through the same gate
job on the macOS build host (the 2093 pattern): the job worktree regenerated it, the file
was copied back over ssh, and the diff was verified to keep the three
windows-targets entries and add no zlib-rs.

## Evidence — gate jobs on the macOS build host (macOS gate host)

| Job | What ran | Verdict |
| --- | --- | --- |
| 2143 73293e7 (1792181229284) | `cargo generate-lockfile` + `cargo deny check advisories` + licenses | advisories ok with zip 9.0.4 (host-only lock, superseded) |
| 2143 73293e7 (1792181487103) | `cargo update -p zip --precise 9.0.4` + deny advisories/licenses; windows-targets 3, zlib-rs 0 | passed; the committed lock's provenance |
| 2143 957cbe8 (1792181517581) | `make lint` | refused: bootstrap-lint-tools has no Darwin-arm64 archive (2137) |
| 2143 65f9965 (1792181557211) | lint sub-checks via `bash -c` | failed only on rustfmt shape in update.rs; fixed in 65f9965's follow-up |
| 2143 65f9965 (1792181589175) | lint sub-checks | died at `with-build-identity` importing tomllib (macOS build host CLT python 3.9.6) |
| 2143 65f9965 (1792181622052) | lint sub-checks, wrapper via python3.13 | clippy 1.95.0 failed on pre-existing code only: `manual fold` at src/mcp/shell.rs:1267, `while_let_loop` at src/sources/opencitations.rs:54. Neither file is touched here; ci.yml pins 1.93.1 where neither fires. Ruled out of scope for this ticket; toolchain drift for 2137 |
| 2143 65f9965 (1792181706452) | `make test` | refused at with-build-identity tomllib (same python 3.9.6 wall) |
| 2143 65f9965 (1792181720333) | `make test` with a python3 shim | archive built `--locked` clean; `tools/run-offline` refuses on Darwin by design ("Linux CI is authoritative") |
| 2143 65f9965 (1792181815429) | nextest from the archive (1817 run before fail-fast) | 1816 passed; the one failure is the gencc subprocess test below |
| 2143 1216658 (1792181933445) | gencc test alone on origin/main's SHA | same failure at src/entities/gene/gencc/tests.rs:622 — pre-existing on macOS, not this change |
| 2143 65f9965 (1792182045199) | zip-affected tests, full no-fail-fast suite | 27/27 update + supplementary tests pass; suite 4258 passed, 19 failed — all in the Darwin classes below |
| 2143 65f9965 (1792182213154) | quality ratchet | refused: check-quality-ratchet.sh:28 unbound empty array under `set -u` on macOS bash 3.2 (fixed in bash 4.4) — 2137 class |
| 2143 65f9965 (1792182240732) | ruff 0.13.2 (uvx-pinned), python contracts, mkdocs | ruff clean; pytest 1112 passed / 33 failed (all Linux-machinery: bubblewrap isolation, fixture-supervisor, publish scripts); mkdocs not reached |

Passing macOS build host pieces: tracked-text, capture receipts, MCP registry, cargo fmt,
clippy on this diff, cargo deny licenses, cargo deny advisories, ruff 0.13.2,
`--locked` compile of lib+bin+every test target, 27/27 zip-affected tests,
4258/4277 suite (19 failures all pre-existing Darwin, proven for the gencc
one on main's SHA and structural for the rest).

Darwin/2137-class refusals on the macOS build host, none in this diff: no pinned
bootstrap-lint-tools archive (shellcheck/actionlint, so check-shell-workflows
and the full `make lint` cannot run); with-build-identity needs python ≥3.11
(macOS build host CLT ships 3.9.6; a python3.13 shim made it run); run-offline refuses by
design; clippy on the macOS build host is 1.95.0 vs ci.yml's pinned 1.93.1; fixture-supervisor
needs bash ≥4.2 (`[[ -v ]]`) and reads /proc; check-quality-ratchet.sh trips
bash 3.2 empty-array `set -u`; the 19 Rust and 33 Python failures all sit in
those classes.

## Landing bar

ci.yml run 38960623859 on tip 65f996506be3625b104359922e58717987a45ab8:
completed success — changes, release-checks, repository-contracts,
generated-sources, build, spec-lane, rust-tests, lint, python-lane all
success; full-features, windows-contracts, stress-lane, release-panic are
main-only lanes and run at landing (the lock keeps the windows-targets
entries they need).

## Advisory

No advisory ignore was added; deny.toml is untouched. `cargo deny check
advisories` reports ok with zip 9.0.4 in the lock. The pre-existing
`constant_time_eq` license-exception warning (dead entry since the 2072
feature trim, already present on main) is unrelated and left for its own
ticket.
