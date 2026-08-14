---
flow: build
priority: 9
---
# Make adverse-event sections format- and source-aware

`get adverse-event` accepts sections, but JSON ignores them and device
Markdown ignores them. Unknown sections can therefore succeed, and callers
cannot rely on the requested projection. The section contract must be the same
at the CLI and MCP boundaries.

Validate the section vocabulary before provider contact. This zero-contact
guarantee applies to unknown names only: whether a syntactically valid request
is FAERS-appropriate cannot be known until the numeric report ID has been
resolved, which may probe FAERS and then device data.

A FAERS request with no sections or `all` keeps the current full Markdown and
JSON contracts unchanged. In particular, full JSON remains the tagged
`{"type":"faers","data":{...},"_meta":{...}}` envelope with its existing
entity fields, evidence, next commands, and section sources.

A subset JSON response keeps that same top-level envelope. Its `data` always
contains `report_id` and `drug`, then includes only the requested data keys:

- `reactions` adds `reactions`.
- `outcomes` adds `outcomes`.
- `concomitant` adds `concomitant_medications`.
- `guidance` adds no `data` key; its commands belong in metadata.

Every selected array is present even when empty. Unselected arrays and the
full-report fields `patient`, `reporter_type`, `reporter_country`, `indication`,
`serious`, and `date` are omitted. Multiple sections form the union in the
canonical order above, independent of request order or duplicates. Implement
this as a typed projection; do not delete fields from serialized JSON.

Subset `_meta.evidence_urls` retains the existing OpenFDA evidence. Its
`section_sources` filters the existing provenance to the selected data-section
keys `reactions`, `outcomes`, and `concomitant_drugs`, retaining existing
presence semantics; it never includes `overview`, and guidance-only therefore
uses an empty list. Its `next_commands` is empty unless `guidance` is selected.
Guidance uses one shared command builder for Markdown and JSON, in this order:

1. `biomcp drug adverse-events <quoted drug>`
2. `biomcp get drug <quoted drug>`
3. `biomcp drug trials <quoted drug>`
4. `biomcp search disease --query <quoted indication>` only when a non-empty
   indication exists

Subset Markdown renders the title/report identity and only the selected
section bodies. It retains evidence URLs, but suppresses the automatic section
navigation and generic related-command footer. When guidance is selected it
renders exactly the shared guidance commands above. This makes guidance-only
and multi-section results agree across formats without changing full output.

The named section vocabulary is FAERS-only. If an ID resolves to a device
report, any syntactically valid section request, including `all`, returns a
clear error after source resolution instead of ignoring it. Do not claim or
test zero provider contact for this case. An unsectioned device request remains
unchanged. Adverse-event batch sections remain unsupported and the public
batch reference must say so explicitly.

## Done when

- Unknown sections fail before retrieval and known FAERS subsets produce the
  exact bounded projection above.
- Full and `all` preserve the existing JSON schema and metadata; subset JSON
  fields, commands, and provenance correspond to the same requested sections
  as Markdown.
- Device section requests fail clearly, while an unsectioned device report remains unchanged.
- Raw and typed MCP calls inherit the same behavior as the terminal CLI.
- Batch help and the static batch reference no longer imply adverse-event
  section support.

## Authorized test changes

The design may add or restate assertions in `src/cli/adverse_event/tests.rs`,
`src/render/markdown/adverse_event/tests.rs`, `src/cli/system/tests.rs`,
`src/cli/list/tests/pages.rs`,
`src/cli/tests/next_commands_json_property/pathway_adverse_event.rs`,
`tests/rmcp_client_contract.rs`, and focused CLI process contracts under
`tests/`. Existing assertions that protect unsectioned/full and batch JSON
output must not be weakened.
