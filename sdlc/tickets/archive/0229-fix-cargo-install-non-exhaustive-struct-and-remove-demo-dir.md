---
flow: quickfix
priority: 9
---
# Fix cargo install non-exhaustive struct and remove demo dir

`cargo install --path .` fails with `E0639: cannot create non-exhaustive struct` at `src/mcp/shell.rs:304`. The installed binary at `~/.cargo/bin/biomcp` is stuck at 0.8.20 while `target/release/biomcp` reports 0.8.21. This blocks user install via PyPI and local `cargo install`. Additionally, the `demo/` directory still exists and causes 1 contract test failure.

Completed under March on 2026-04-17, as March ticket 229. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/229-fix-cargo-install-non-exhaustive-struct-and-remove-demo-dir
