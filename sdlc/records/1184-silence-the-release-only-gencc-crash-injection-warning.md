---
flow: quickfix
priority: 8
---

# Silence the release-only GenCC crash-injection warning

The GenCC crash-injection helper now explicitly consumes both of its
arguments after the debug-only injection block. Release builds no longer warn
that the injected error is unused, while debug crash markers, exit behavior,
and injected error returns are unchanged.

## Evidence

Independent design and code reviews accepted the one-line repair with no
findings. Two `cargo build --release --no-default-features` runs completed with
zero warnings. The focused GenCC suite passed twice with 49 tests passing and
one intentional subprocess-helper ignore. `make lint`, `make test`, and
`make spec` passed on the frozen candidate under Node 22.

## Boundary

Only the release-build discard expression changed. No CLI, MCP, provider,
cache, public API, or runtime behavior changed.
