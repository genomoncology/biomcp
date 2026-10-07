# 2028 — Adopt the shared OncoKB annotation response

Status: OPEN.
Milestone: 1.0
Component: backend
Owner: Root owns the consumer queue and paired producer0716. A fresh builder will own only2028 after producer acceptance.
Phase: Consumer BUILD released against accepted producerc46b3e6cfaf9659a8add076734b44afb4bccd002, BioData0.0.43.

## Outcome

Use the shared direct OncoKB response in the actual variant helper. Preserve CLI JSON/Markdown and raw MCP forwarding, request spelling attempts, authentication, transport, error categories, therapy ordering and output policy. Retire five local source schema types and the successful decoder after actual caller checks pass.

## Reviewed contract

The producer0716 [brief](../../../../repos/biodata/sdlc/planning/0716-oncokb-annotation-response.md) holds the unchanged paired contract. Fresh Medium design review accepted78c0fd8c650b490c2bff698bde45bf9ea74574d9 after correcting positional defaults and ignored numeric admission. This ticket assigns its existing consumer contract; it introduces no API or scope change. Base: e9007e1def8f6206ddf1a72948ab4cdf3c8b702f.

Changes: `src/sources/oncokb.rs`, `src/sources/oncokb/tests/parsing.rs`, `src/entities/variant/get.rs`, `src/entities/variant/get/tests.rs`, `tests/test_oncokb_response_adoption.py`, `tests/test-file-caps.json`, `Cargo.toml`, `Cargo.lock`, `tools/biodata-1.0-focused.toml`, `tools/rust-source-size-inventory.json`, `tools/check-biodata-boundary.py`, `tests/test_biodata_boundary.py`, `tests/test_source_package_boundary.py`, `tests/test_biodata_branch_workflow.py`, this ticket and its owning record.

## Evidence and done

Reuse existing source parsing, authentication/request construction, helper truncation and Markdown owners. Experiment369 records the token-gated helper and source/interpretation separation. The accepted0716 brief lists literal expected outputs and prior completed boundaries. Keep one actual helper success owner and one failure owner through existing facilities. Producer owns source admission; do not copy its edge matrix. Move or retire displaced test claims after their replacement passes. Record compilation and affected test costs. No whole maintenance gate, hosted jobs, live source access, private payload copies, source-rights changes or deletion.

Root reconciles maintenance before consumer BUILD and exact CODE review, obtains fresh read-only review, verifies relevant checks, lands on biodata/biomcp-1.0 and pushes. Whole pair completes when the accepted producer is pinned, the actual helper preserves reviewed outputs, and displaced schema ownership is retired. Producer API alone does not close either outcome. BUILD waits for the exact accepted producer pin, not whole-pair COMPLETE; Root releases it explicitly.

Reviews: accept

## BUILD release

Fresh Medium CODE review accepted producerbd6d0ae76609fe2afb55d6f7fc019013a5a54a82. Producer landedc46b3e6cfaf9659a8add076734b44afb4bccd002 and is pushed. Admission, lint, package spec and ordinarytest passed. The test run spans only an inventory correction, with unchanged runtime/tests, as its owning record states. Root reconciles current maintenancec3b339c9 before this consumer build. A fresh builder owns only2028 in the existing ticket worktree. Root owns independent CODE review and paired landing. Cargo uses the declared exact Git pin through its existing host cache; no path patch or source-policy change.
