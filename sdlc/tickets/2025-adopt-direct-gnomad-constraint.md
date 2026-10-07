# 2025 — Adopt the complete direct gnomAD gene constraint record

Status: OPEN.
Owner: Root; assigned Codex design writer.
Phase: DESIGN. Fresh paired review pending. Root releases BUILD separately.
Component: backend

Milestone: 1.0

## Outcome

Transfer every selected direct gene constraint field through BioData0712, shared runtime storage and borrowed encoding into existing native, CLI and MCP callers. Retire the local value models after adoption. The [paired consumer design](../planning/direct-gnomad-constraint-2025/design.md) specifies retained policy, behavior owners and later file claims. This ticket authorizes design preparation only.

## Evidence

- Starts from: dedicated target `9747e3f05d74881235a5a8d1a072938758856901`, accepted BioData `25f0c7fd3cee947230231aae0e3ed3aa960ecd46` version `0.0.38`, completed direct population2015 and incorporated maintenance `37631c3574703755bf742e1e8459bc0d77522220`. Actual sources, both retrieval strategies, serializers, renderers and records still own gene constraint locally. Filtered 1.0/backend queue has no duplicate design. Fresh origin/main `1c774def2` adds records/tickets only; Root owns reconciliation.
- Keeps: complete transcript/pLI/LOEUF/mis_z/syn_z values, source normalization/nulls, target omission, GraphQL policy, provider provenance, metadata, section selection/outcomes, cache/retries/body limits/timeouts, serial/ParallelTop execution and existing JSON/Markdown privacy/display behavior.
- Changes: paired producer0712 record/decoder/getters/borrowed target; direct storage under product metadata; removal of duplicate source models and field-copy mapper after real caller proof. No new scaffolding, checker or provider acquisition.
- Proof: existing fixtures as unverified compatibility examples; one actual caller owner for direct source, native retrieval strategies and CLI/raw/typed MCP; distinct producer admission/encoding owner; retain product-policy and renderer tests with minimal setup changes. Independent expected values and explicit no-call/empty/unavailable/provenance assertions.
- Defers: whole Gene ownership, other source batches, biological interpretation, publication/release, implementation and deletion. No parser run, build, provider call, private production data, CI, Pi, CLI worker or nested agent occurs in DESIGN.

## Review

Global number reserved by `pm ticket new` in an isolated 1.0 checkout. Producer0712 uses its own allocated ticket and branch. Push DESIGN with `[skip ci]`; Root assigns fresh paired review and supplies BUILD authority after acceptance. No implementation or independent ACCEPT is claimed.
