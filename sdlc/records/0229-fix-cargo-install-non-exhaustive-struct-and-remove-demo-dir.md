---
base: 5ab6a111943ccc43128707c07f87e8d626ae147b
head: 0099e4e6c5709ec24c6f51e02bcba81483106bd9
---
`cargo install --path .` fails with `E0639: cannot create non-exhaustive struct` at `src/mcp/shell.rs:304`. The installed binary at `~/.cargo/bin/biomcp` is stuck at 0.8.20 while `target/release/biomcp` reports 0.8.21. This blocks user install via PyPI and local `cargo install`. Additionally, the `demo/` directory still exists and causes 1 contract test failure.

Imported from March ticket 229. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/229-fix-cargo-install-non-exhaustive-struct-and-remove-demo-dir
