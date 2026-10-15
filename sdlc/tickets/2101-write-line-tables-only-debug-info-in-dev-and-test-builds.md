# 2101 — Write line-tables-only debug info in dev and test builds

Status: OPEN.

Milestone: 0.9.3

## Outcome

Dev and test builds write line tables instead of full debug info, so they compile, link and fill the target folder faster and smaller, while backtraces still name files and lines. A `debugging` profile gives full debug info when someone needs a debugger.

## Evidence

- `Cargo.toml` sets only `[profile.release]`. Every dev and test build, local, CI and the spec build (`SPEC_PROFILE` defaults to `dev`), writes full debug info for the crate and every dependency.
- Cargo's build-performance guide recommends `debug = "line-tables-only"` for dev builds and no debug info for dependencies. Line tables keep file and line numbers in panics and backtraces.
- The shared build hosts carry several targets at once, and disk is limited on the macOS gate host.

- Starts from: `Cargo.toml`, `Makefile` (`SPEC_PROFILE`), `scripts/run-specs.sh`.
- Decisions:
  - `[profile.dev]` and `[profile.test]`: `debug = "line-tables-only"`.
  - `[profile.dev.package."*"]` and `[profile.test.package."*"]`: `debug = false`.
  - `[profile.debugging]`: `inherits = "dev"`, `debug = true`.
  - Release stays as it is.
- Changes: claims `Cargo.toml` `CHANGELOG.md` `docs/reference/**` (only if a page names build profiles).
- Proof: on the Linux gate host, at main before and the branch after, record in the review: one cold `cargo nextest run --no-run --no-default-features` wall time, one warm rebuild after touching `src/main.rs`, and the target folder size after each. The routine gates pass unchanged; a panic backtrace in a test still names the file and line. Land only if no measure gets worse.
- Order: safe alongside anything; shares no file with the open speed tickets except `CHANGELOG.md`.
