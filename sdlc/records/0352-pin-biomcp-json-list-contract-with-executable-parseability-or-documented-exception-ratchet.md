---
base: c31c2a9be55a5f00998c13c8d59368d2a6e99a04
head: e326f6617fc1706795da21cbfe1cbb1a8ffd11d8
---
The global `--json` flag's help text says "Output as JSON instead of Markdown (except biomcp cache path, which stays plain text)". `biomcp --json list` and `biomcp --json list gene` are a second undocumented exception: both exit 0 and print Markdown. The 348 reviews (outside-in, code-review, architecture) all surfaced this as the highest-impact contract gap visible to script/agent callers — agents commonly pass `--json` globally, and there is no parser-level signal that the flag was silently ignored. Code review traced the cause to `src/cli/outcome.rs` routing `Commands::List` directly to `crate::cli::list::render(entity.as_deref())` without consulting the JSON flag.

Imported from March ticket 352. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/352-pin-biomcp-json-list-contract-with-executable-parseability-or-documented-exception-ratchet
