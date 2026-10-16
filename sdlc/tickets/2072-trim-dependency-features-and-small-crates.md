# 2072 — Trim dependency features and small crates

Status: COMPLETE.

Milestone: 0.9.3

Depends on: 2076

Reviews: revision cffbdfb, accept

Reviews: revision cffbdfb, accept

## Outcome

Routine builds compile no zstd C library, the shipped binary has no tokio test clock, the AlphaGenome client compiles no gRPC server, and BioMCP no longer depends on `fs2`, so Windows builds pull no `winapi 0.3`.

## Evidence

- `zip 0.6` with default features pulls bzip2, AES, PBKDF2 and a second zstd, so the `zstd-sys` C build costs 26.8 s per cold routine build and up to 97 s in release. Both zip call sites read deflate archives (`sdlc/designs/2026-10-13-0-9-3-dependencies-and-dead-code.md`, step 1).
- tokio's `test-util` is used only under `#[cfg(test)]`; `tokio-stream` is used only by AlphaGenome yet is not optional; tonic takes its default `transport` feature, server included (steps 2 and 3).
- `fs2` alone pulls `winapi 0.3` on Windows (step 4).
- PM decision 2026-10-14: replace only `fs2` among the five small crates. Changing the `md5` download key would orphan files in `downloads/`, which `cache clear` never touches; replacing `humantime` and `bytesize` means rewriting the user-facing parse and print rules of `--max-age` and `--max-size`; all three together save about 0.7 s of build time. scraper's features and `getrandom` stay with 2078, because `readability-rust` asks for scraper's defaults and 2078 replaces that stack. `toml 1.x` is out of 0.9.3 unless a measured ticket is filed.

- Starts from:
  - `Cargo.toml` `[dependencies]`: `zip = "0.6"` (defaults bring bzip2, AES and zstd 0.11); tokio's feature list, which includes `"test-util"`; `tokio-stream = "0.1"`, not optional, though only `src/sources/alphagenome.rs` (lines 4 and 82) uses it and that module compiles only with the `alphagenome` feature (`src/sources/mod.rs:438`); `tonic` with its default `transport` feature; `fs2 = "0.4"`.
  - The two zip call sites: `src/cli/update.rs`, `extract_binary_from_zip` (line 105), which has no test; `src/sources/europepmc.rs:493`, whose tests in `src/sources/europepmc/tests/parsing.rs` build zips with `zip::ZipWriter`.
  - The fs2 call sites. Production: `src/cli/article/session.rs`, `src/cache/clear.rs`, `src/cache/limits.rs` (`available_space` and `total_space`), `src/cache/migration.rs`, `src/cache/provider_capture.rs`, `src/sources/gencc/store.rs`. Tests: `src/entities/article/search/tests/deadline.rs`, `src/sources/gencc/tests.rs`.
  - `tests/test_alphagenome_proto_generation.py:39`, which asserts the exact `alphagenome = [...]` feature line.
- Keeps: every user feature; the same lock behavior (std file locks use flock on Unix and LockFileEx on Windows, as fs2 does); `biomcp cache` and health output byte-identical; `md5`, `humantime`, `bytesize` and `toml`.
- Changes: claims `Cargo.toml` `Cargo.lock` `tests/test_alphagenome_proto_generation.py` `src/cli/update.rs` `src/cli/article/session.rs` `src/cache/clear.rs` `src/cache/limits.rs` `src/cache/migration.rs` `src/cache/provider_capture.rs` `src/sources/gencc/store.rs` `src/sources/gencc/tests.rs` `src/entities/article/search/tests/deadline.rs` `CHANGELOG.md` `tests/test_docs_changelog_refresh.py`.
  - `zip = { version = "0.6", default-features = false, features = ["deflate"] }`.
  - Remove `"test-util"` from tokio's features and add `tokio = { version = "1", features = ["test-util"] }` under `[dev-dependencies]`.
  - `tokio-stream = { version = "0.1", optional = true }`, with `"dep:tokio-stream"` in the `alphagenome` feature; update the assertion in `tests/test_alphagenome_proto_generation.py`.
  - `tonic = { version = "0.12", default-features = false, features = ["codegen", "prost", "channel", "tls", "tls-roots"], optional = true }`.
  - Replace fs2 with std's `File::lock`, `lock_shared`, `try_lock`, `try_lock_shared` and `unlock` (stable since Rust 1.89), mapping `TryLockError::WouldBlock` onto the existing `io::ErrorKind::WouldBlock` branches. In `src/cache/limits.rs`, read free and total space through `libc::statvfs` on Unix and `GetDiskFreeSpaceExW` on Windows from the `windows-sys` features already listed.
  - A unit test in `src/cli/update.rs` builds a deflate zip with `zip::ZipWriter` and extracts the binary through `extract_binary_from_zip`.
  - One Internal changelog bullet.
- Proof: the values below were measured on main at ca54283f0, before 2070 and 2076 edited `Cargo.toml`; the developer re-measures each "today" value on current main before the first push and records it on this ticket:
  - `cargo tree --no-default-features -e normal,build | grep -c zstd-sys` prints 1 today and 0 after.
  - `cargo tree -d -e normal,build` lists axum 0.7.9, axum-core 0.4.5, matchit 0.7.3 and zstd 0.11.2 today and none of them after. cipher 0.4 and inout 0.1 stay because unpdf needs them.
  - `cargo tree -e features -i tokio -e normal | grep -c test-util` prints 0 after.
  - `cargo tree --target x86_64-pc-windows-msvc -i winapi -e normal` shows only `winapi v0.3.9 ← fs2` today and nothing after.
  - The new update zip test passes, with the existing Europe PMC zip, cache, health and GenCC lock tests.
  - `tools/measure-0-9-3-baseline <clone> --only dep-counts,cold-dev-build` against the baseline (338 crates normal and build, 349 with dev, cold dev build 250.9 / 260.3 s); a trial manifest edit drops the routine tree by about 12 crates.
  - After `make test`, `make spec` compiles nothing, which proves moving test-util into dev-dependencies did not split 2070's single build.
- Defers: scraper's features, getopts and getrandom (2078); the tonic upgrade (2077); toml 1.x.
- Order: after 2070, which also edits `Cargo.toml` and the same assertion line in `tests/test_alphagenome_proto_generation.py`. After 2076, which edits `Cargo.toml` and deletes `tools/rust-source-size-inventory.json`, so this ticket no longer edits that file.
