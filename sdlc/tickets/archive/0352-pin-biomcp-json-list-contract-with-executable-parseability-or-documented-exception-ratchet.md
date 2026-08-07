---
flow: build
priority: 7
---
# Pin biomcp --json list contract with executable parseability or documented exception ratchet

The global `--json` flag's help text says "Output as JSON instead of Markdown (except biomcp cache path, which stays plain text)". `biomcp --json list` and `biomcp --json list gene` are a second undocumented exception: both exit 0 and print Markdown. The 348 reviews (outside-in, code-review, architecture) all surfaced this as the highest-impact contract gap visible to script/agent callers — agents commonly pass `--json` globally, and there is no parser-level signal that the flag was silently ignored. Code review traced the cause to `src/cli/outcome.rs` routing `Commands::List` directly to `crate::cli::list::render(entity.as_deref())` without consulting the JSON flag.

Completed under March on 2026-04-29, as March ticket 352. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/352-pin-biomcp-json-list-contract-with-executable-parseability-or-documented-exception-ratchet
