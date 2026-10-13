# 2072 — Trim dependency features and small crates

Status: OPEN.

Milestone: 0.9.3

## Outcome

BioMCP compiles no dependency code it never calls: no zstd C build in routine builds, no test clock in the product, no gRPC server, and five small crates replaced by std or crates already present.

## Evidence

- `zip 0.6` with default features pulls bzip2, AES, PBKDF2 and a second zstd, so the `zstd-sys` C build costs 26.8 s per cold routine build and up to 97 s in release. Both zip call sites read deflate archives (`sdlc/designs/2026-10-13-0-9-3-dependencies-and-dead-code.md`, step 1).
- tokio's `test-util` is used only under `#[cfg(test)]`; `tokio-stream` only by AlphaGenome yet is not optional (step 2).
- tonic's `router` and `server` features and scraper's default `main` feature are unused (step 3).
- `fs2`, `md5`, `getrandom`, `humantime` and `bytesize` each have one to five call sites that std, `sha2`, `uuid` or about 100 lines replace; `fs2` alone pulls `winapi 0.3` on Windows (step 4).

- Starts from: `sdlc/designs/2026-10-13-0-9-3-dependencies-and-dead-code.md`, steps 1 to 4 and 8.
- Keeps: every feature users have, `biomcp cache` and health output byte-identical.
- Changes: `zip` with only `deflate`; tokio `test-util` in dev-dependencies; `tokio-stream` under the `alphagenome` feature; tonic without `router` and `server`; scraper without defaults; replace the five small crates; try `toml 1.x` and keep it only if it compiles faster.
- Proof: `cargo tree --no-default-features -e normal,build | grep -c zstd-sys` prints 0; `cargo tree -d` loses axum 0.7, axum-core 0.4, matchit 0.7, getopts; `cargo tree --target x86_64-pc-windows-msvc -i winapi` is empty; a fixture test extracts a deflate zip through both call sites; cache and health output tests pass unchanged; crate count and cold build time before and after.
- Defers: the tonic upgrade (2077) and the HTML stack (2078).
