---
flow: build
priority: 10
deps: ["0951"]
---
# Keep routine specs from launching test-owned gates

`spec/surface/cli-contract-ratchet.md` starts pytest from inside `make spec`.
Those source-policy tests already run under `make test`, including a complete
quality-ratchet invocation. The spec lane therefore repeats another gate rather
than exercising a distinct product behavior.

## Test contract

Classify every executable routine Markdown block as product behavior, prepared
artifact evidence, or an accidental invocation of another gate. Add a source
ratchet that rejects pytest, nextest, `make test`, `make lint`, and complete
quality-checker invocations from routine pages and their helpers.

## Done when

- The CLI surface ratchet remains directly covered under `make test` and is no
  longer launched by `make spec`.
- Routine Markdown pages contain product-facing CLI/MCP behavior or bounded
  inspection of preparation-owned evidence, not nested test/lint gates.
- The ratchet parses executable blocks and helper commands without matching
  prose or expected-output literals.
- A fixture for every forbidden nested gate fails with a path and line.
- The routine spec result set retains all distinct product assertions.

## Authorized test changes

Design commits may move source-policy assertions from Markdown to their owning
Python/Rust contract tests, restate the affected routine pages, and extend the
spec source ratchet. Product-facing assertions may not be dropped.

The src line ceiling may not rise.
