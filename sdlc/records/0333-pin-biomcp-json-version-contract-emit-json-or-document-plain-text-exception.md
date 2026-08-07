---
base: 19126a6ba46c46d98799740e0ce5b7dc5f10cdf7
head: 4e47906a7621280992df3dbc1413f158e9056315
---
Top-level `biomcp --help` documents `--json` as "Output as JSON instead of Markdown (except biomcp cache path, which stays plain text)". `target/release/biomcp --json version` emits plain text instead of JSON. Operationally low-impact, but it is a public contract inconsistency and it represents an unratcheted exception list.

Imported from March ticket 333. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/333-pin-biomcp-json-version-contract-emit-json-or-document-plain-text-exception
