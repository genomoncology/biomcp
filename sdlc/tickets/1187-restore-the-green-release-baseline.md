---
flow: quickfix
priority: 10
deps: []
---

# Restore the green release baseline

## Outcome

The current St. Jude talk remains published and packaged while the canonical
test lane returns to green and the source package remains exactly 1,300 files.

## Current facts

On `d8d1f5e4`, `cargo package --list --allow-dirty --locked --offline` reports
1,302 paths. The two package tests reject that count, the full agent index omits
the new talk page, and the README landing-copy test reads the linked talk image
inside the bounded `What is BioMCP?` section and rejects it.

Two packaged Rust paths are no-op leftovers: the unreferenced
`src/transform/trial/tests/ticket_1132.rs` contains only a comment, and
`src/cli/benchmark/run/tests/render.rs` is an empty test module referenced only
by its adjacent `#[path] mod tests` declaration at runtime. The benchmark
structure contract also inventories that empty path.

## Scope

Move the existing README talk section below the Quick start command block, add
the talk page to `docs/llms-full.txt`, and remove the two no-op Rust paths plus
the empty module declaration and its structure-contract inventory entry.
Preserve the talk page and image in the package. Do not raise the package
ceiling or change runtime behavior. Stage or commit the tracked deletions before
running the quality ratchet because its worktree reader expects deleted Rust
paths to be reflected in the index.

## Acceptance

The two source-package tests, the full-agent-index test, the README landing-copy
test, and `benchmark_internal_harness_split_files_exist_with_doc_headers` pass.
The package contains exactly 1,300 files, both talk assets remain listed, the
zero-coupling check passes, and routine repository gates pass.

## Dependencies

None. This repairs the failing main baseline before feature work resumes.

## Complexity

- Contract score: 1
- State and timing score: 0
- Reach score: 1
- Proof score: 1
- Cost of error score: 1
- Total: 4
- Minimum level floor: none
- Final level: 2
- Reasons: one existing package limit spans package and documentation contracts;
  focused deterministic tests cover every changed surface.
- Selected model: GPT-5.6 Luna, high reasoning

## Review

- Design review: accepted after the benchmark inventory and staged-deletion
  constraints were made explicit and the complexity score was corrected.
- Code review: accepted. The reviewer confirmed the 1,300-path package, both
  retained talk assets, all four repaired contracts, and no runtime change.
