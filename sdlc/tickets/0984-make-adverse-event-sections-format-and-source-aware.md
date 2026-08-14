---
flow: build
priority: 9
---
# Make adverse-event sections format- and source-aware

`get adverse-event` accepts sections, but JSON ignores them and device
Markdown ignores them. Unknown sections can therefore succeed, and callers
cannot rely on the requested projection. The section contract must be the same
at the CLI and MCP boundaries.

Validate section names before provider contact. A FAERS request with no
sections or `all` keeps the current full output. A subset request keeps report
identity and emits only its selected reactions, outcomes, concomitant drugs,
or follow-up guidance in both Markdown and JSON. Use a typed JSON projection;
do not delete fields from an already serialized entity.

The named section vocabulary is FAERS-only. If an ID resolves to a device
report, any section request, including `all`, returns a clear error instead of
ignoring it. Adverse-event batch sections remain unsupported and the public
batch reference must say so explicitly.

## Done when

- Unknown sections fail before retrieval and known FAERS subsets produce distinct bounded output.
- JSON subset fields and metadata correspond to the same requested sections as Markdown.
- Device section requests fail clearly, while an unsectioned device report remains unchanged.
- Raw and typed MCP calls inherit the same behavior as the terminal CLI.
- Batch help no longer implies adverse-event section support.

## Authorized test changes

The design may add or restate assertions in `src/cli/adverse_event/tests.rs`,
`src/render/markdown/adverse_event/tests.rs`, `src/cli/system/tests.rs`,
`tests/rmcp_client_contract.rs`, and focused CLI process contracts under
`tests/`.
