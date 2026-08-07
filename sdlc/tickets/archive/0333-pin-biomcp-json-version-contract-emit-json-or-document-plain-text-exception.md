---
flow: quickfix
priority: 6
---
# Pin biomcp --json version contract: emit JSON or document plain-text exception

Top-level `biomcp --help` documents `--json` as "Output as JSON instead of Markdown (except biomcp cache path, which stays plain text)". `target/release/biomcp --json version` emits plain text instead of JSON. Operationally low-impact, but it is a public contract inconsistency and it represents an unratcheted exception list.

Completed under March on 2026-04-28, as March ticket 333. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/333-pin-biomcp-json-version-contract-emit-json-or-document-plain-text-exception
