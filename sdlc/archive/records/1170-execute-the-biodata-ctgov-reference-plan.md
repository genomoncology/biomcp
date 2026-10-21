---
flow: build
priority: 1
---
# Execute the BioData ClinicalTrials.gov reference plan

## Outcome

The dedicated `biomcp get trial NCT... references` path builds its request with BioData `0.0.5`, sends that plan through BioMCP's existing HTTP stack, and gives the untouched provider bytes to BioData for identity-checked parsing. BioMCP renders references from BioData's single plan-bound section result. The broader `all`, mixed-section, and other section paths retain their current fields and behavior until their areas migrate.

## Current facts

At BioMCP `a211ada71749231f39b735850467726d79325797`, `ClinicalTrialsClient::get_plan` and `build_get_fields` own every CTGov detail request. `ClinicalTrialsClient::send` already returns bounded `Vec<u8>` before `decode_get_response` parses it. `entities::trial::get` converts the legacy `CtGovStudy` and forces a requested but absent reference section to an empty list.

BioData `9c470d5d3aad97b34bd4be71924475c03973a331` exports version `0.0.5` with a validated credential-free `ClinicalTrialsGovApiV2DetailPlan`, exact-byte `ClinicalTrialsGovApiV2Response`, and four-state `ClinicalTrialSection<T>`. Its plan-bound response accepts identification plus references without unrelated overview modules. It owns one exact-byte capture and one reference collection. Missing, null, and exact-empty optional reference members become absent shared values. Whitespace-only strings remain available for the product's display policy. Its reference plan selects the same eighteen base fields and three reference fields used by the current BioMCP references request. This BioData revision adds the NCI field-selected plan without changing the accepted ClinicalTrials.gov API.

The BioData plan does not yet describe contacts, locations, outcomes, arms, eligibility, or posted documents. Sending its field query for `all` would lose current functionality. This ticket therefore migrates only the dedicated references request. The existing shared trial and product renderer cannot yet show a visible difference between a requested absent list and a requested explicit empty list. BioData still keeps that distinction at the provider-response boundary.

## Scope and decisions

Update the exact Cargo Git pin and lockfile to BioData `9c470d5d3aad97b34bd4be71924475c03973a331` at version `0.0.5`. Update the live package-development contract for that exact revision without weakening the 1,300-file ceiling or private-content boundaries.

Use the new path only when the normalized request contains at least one section token and every meaningful token is `references`. Ignore blank tokens and the existing `--json` and `-j` markers during that decision. Keep `all`, `references outcomes`, and every other mixed request on the legacy path. Construct the BioData plan after BioMCP validates the user input. Translate its relative path and field query into the existing BioMCP `RequestPlan`. Execute it through the current base-URL policy, cache mode, retries, timeout, bounded body reader, HTTP-status mapping, and source error context. Do not add credentials or HTTP behavior to BioData.

Inspect HTTP status before BioData parses the body. Preserve the current `NotFound` value and recovery text for HTTP 404. Pass every other non-success status through the current ClinicalTrials.gov status decoder. This rule applies even when the body contains valid JSON. Pass the same successful response `Vec<u8>` directly to `ClinicalTrialsGovApiV2Response::parse`. Do not parse and reserialize before BioData sees it.

Map `malformed_json`, `unsupported_json`, `invalid_projection`, and `identity_mismatch` to `BioMcpError::Api` with the ClinicalTrials.gov source, a fixed message that includes only the stable BioData code, and `SourceContext::retry`. Map `json_resource_limit` to the same sanitized `BioMcpError::Api` shape with `SourceContext::narrow`. These mappings keep product code `api`, the existing source label, and the named recovery action. They must not include response bytes, requested identity, returned identity, or BioData's display text.

The product may parse the same original bytes through the temporary legacy CTGov decoder for unmigrated overview fields. It must take reference values from the BioData response only. Omit a reference row when its citation is missing, null, or blank after trimming. Trim the optional PMID and reference type exactly as the current product conversion does. Preserve source order for retained rows. A private product wrapper may clone the shared reference value for current serialization. It must not store duplicate PMID, citation, or type fields. Keep the existing legacy reference extraction only for `all` and mixed requests, where the BioData plan does not yet own the other requested fields. Delete that extraction and the compatibility wrapper before the checkpoint after arms and eligibility.

Map `Present(values)` to the current ordered product references. Map `Absent` and `Present(empty)` to the current successful empty reference output. Treat `NotRequested` and `Unavailable` as internal contract errors because this path always requests a provider-supported section. Do not silently turn an impossible state into missing data.

Use the existing loopback CTGov process test. Assert the exact normalized path and complete twenty-one-field query. Serve recorded provider bytes directly for populated evidence. Use labelled synthetic bytes for absent, explicit empty, wrong identity, malformed JSON, unsupported shape, resource limit, invalid projection, missing citation, null citation, blank citation, and HTTP failures. Preserve JSON, Markdown, retained reference order, Unicode, 404 behavior, safe errors, `all`, `references outcomes`, and a non-reference detail path. Do not add a provider capture or complete-output hash.

Do not change search, `all`, other detail sections, MCP schemas, rendering formats, provider scope, or release versions. Do not fix unrelated BioMCP bugs in this ticket.

## Acceptance

1. Cargo and the package-development contract pin BioData `9c470d5d3aad97b34bd4be71924475c03973a331` at version `0.0.5` exactly.
2. The dedicated references request uses the BioData path and exact twenty-one-field query through the existing BioMCP transport stack.
3. BioData receives untouched bytes only after a successful HTTP status and before any legacy parse. HTTP 404 retains the current not-found value and recovery text. Other non-success statuses retain the current source, `api` classification, and retry recovery even when their bodies contain valid JSON.
4. The product's dedicated reference result comes only from `ClinicalTrialSection<Vec<ClinicalTrialReference>>`. No duplicate reference field storage appears.
5. Focused tests prove populated, absent, explicit empty, ordered, Unicode, wrong identity, malformed JSON, unsupported shape, resource limit, invalid projection, every documented error mapping, HTTP 404, another non-success HTTP status with valid JSON, missing/null/blank citation filtering, and non-reference behavior. Recorded bytes prove provider content. Synthetic bytes prove validation only.
6. The `all` and `references outcomes` paths retain their current requested fields and output. The ticket records the deletion trigger for their temporary legacy extraction and wrapper.
7. The source package remains at or below 1,300 files. Private-content checks remain active.
8. Independent design and code reviews accept the result. Focused red-green evidence and `make lint`, `make test`, and `make spec` pass.

## Dependencies

BioData tickets 0095 through 0097 provide the request plan, partial response, single reference owner, and optional reference behavior first released in `0.0.4`. BioData `0.0.5` preserves that API and adds the NCI field-selected plan. BioMCP records 1166 and 1169 provide the capability inventory and initial shared reference ownership. Both factory channels remain paused. The manual subagent SDLC owns this work.

## Review

- Design review: rejected at `0.0.2` because the library required unrelated overview modules, retained references twice, left the dedicated-path predicate undefined, underspecified HTTP and parser error mappings, and omitted the product's citation filtering rule. The `0.0.3` re-review found that exact-empty reference members failed before BioMCP could apply its existing display policy. BioData tickets 0096 and 0097 corrected the library blockers. The independent reviewer accepted the `0.0.4` API against BioMCP `a211ada71749231f39b735850467726d79325797`. The implementation now pins additive BioData `0.0.5` because this project supports only the latest version.
- Code review: rejected because the decoder briefly returned both the shared and legacy reference owners, direct four-state conversion tests were incomplete, and the 404 process test did not assert retry guidance. Remediation moved legacy-owner removal into the decoder, added direct tests for all four section states, and asserted the existing 404 recovery text. Independent re-review accepted all three corrections and found no reference-path regression.

## Implementation evidence

- Red: `cargo test --locked biodata_reference_response_returns_one_reference_owner` failed because the decoded legacy study still held `references_module` beside the BioData section result.
- Green after remediation: `cargo test --locked biodata_reference_response_returns_one_reference_owner` and `cargo test --locked product_references_maps_each_section_state` each passed. `uv run --no-sync pytest -q tests/test_ctgov_trial_search_detail_reuse.py` passed all 9 process tests. The worktree-scoped package command `TMPDIR="$PWD/.cache/focused-tmp" uv run --no-sync pytest -q tests/test_source_package_boundary.py` passed all 6 package tests.
- Complete post-remediation gates: `make lint && make test && make spec` passed against BioData `0.0.5`. The run passed all lint checks, 3,158 Rust tests with 30 skipped, 904 Python tests with 3 skipped, strict documentation, and every specification suite. `cargo package --list --allow-dirty --locked --offline | wc -l` returned exactly 1,300 files. Independent re-review reran three focused Rust tests and all nine loopback process tests, checked the package count and diff, and accepted the result.
