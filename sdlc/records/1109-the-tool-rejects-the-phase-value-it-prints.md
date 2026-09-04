---
flow: build
priority: 7
---

# Phase values emitted by either trial source must round-trip as filters

BioMCP intentionally exposes a trial phase as `Option<String>`. CTGov supplies
a list and the converter joins it with `/`; NCI supplies a scalar and the
converter preserves it. The internal filter normalizer uses `Vec<String>` to
carry semantic canonical phases to the two source-specific request builders.
This ticket changes input normalization and advertised inputs, not that public
output shape or its bytes.

Two receipted records demonstrate the defect:

- `testdata/sources/ctgov/search_keytruda_limit3_20260811.json`, trial
  `NCT05431270`, supplies `PHASE1` and `PHASE2`; BioMCP emits
  `PHASE1/PHASE2`, which `normalize_phase` currently rejects.
- `testdata/sources/nci_cts/search_melanoma_20260811.json`, trial
  `NCT05929768`, supplies and BioMCP emits `III`, which the same normalizer
  rejects.

The earlier ticket cited the unreceipted `search_melanoma.json`; it is not the
acceptance evidence. Ticket 1110 described the same NCI defect and is absorbed
here.

## Required normalization vocabulary

Normalize these inputs to the shared canonical vector:

| Accepted input | Canonical vector |
| --- | --- |
| `NA`, `N/A`, `n/a` | `[NA]` |
| `EARLY_PHASE1`, `early_phase1`, `early1` | `[EARLY_PHASE1]` |
| `PHASE1`, `1`, `I` | `[PHASE1]` |
| `PHASE2`, `2`, `II` | `[PHASE2]` |
| `PHASE3`, `3`, `III` | `[PHASE3]` |
| `PHASE4`, `4`, `IV` | `[PHASE4]` |
| `PHASE1/PHASE2`, `1/2`, `I_II` | `[PHASE1, PHASE2]` |
| `PHASE2/PHASE3`, `2/3`, `II_III` | `[PHASE2, PHASE3]` |

Matching remains ASCII-case-insensitive where the existing normalizer is
case-insensitive. Whitespace around the entire value and around `/` is ignored.
Do not interpret arbitrary Roman strings or arbitrary compounds. Duplicate,
reversed, empty, or mixed malformed compounds such as `PHASE1/`, `III/IV`,
`PHASE2/PHASE1`, `I__II`, and `PHASE1/BOGUS` fail atomically; no partial vector
or provider request is produced.

The bounded emitted provider vocabulary is CTGov `NA`, `EARLY_PHASE1`,
`PHASE1` through `PHASE4`, `PHASE1/PHASE2`, and `PHASE2/PHASE3`; and NCI `NA`,
`I`, `I_II`, `II`, `II_III`, `III`, and `IV`. The table accepts every member.
No provider evidence supports another emitted compound in this repository; add
evidence and an explicit mapping before expanding this set.

The combination vocabulary has durable provider evidence. The receipted CTGov
metadata at `testdata/sources/ctgov/field_metadata_20260903.json` documents the
`phases` field as `Phase[]` and states that its permitted two-phase combinations
are phases 1&2 and 2&3. For NCI, the provider-owned public
`NCIOCPL/clinical-trials-search-app` at commit
`ec2f646f602c8585b93a12336ae0944ae4f26f93`,
`src/utilities/formatTrialSearchQueryV2.js`, lines 58-78, constructs its phase
filter with the tokens `i_ii` and `ii_iii`. That source establishes the NCI
combined token vocabulary; its additional scalar-overlap expansion is not
adopted here because this ticket explicitly preserves BioMCP's existing scalar
request behavior.

## Source request semantics

The canonical vector continues through the existing source-specific mapping:

- CTGov sends a single `AREA[Phase]PHASEn` term for a scalar and an `AND` of
  the two area terms for either supported combined vector.
- NCI sends `I`, `II`, `III`, `IV`, or `NA` for scalar canonical values,
  `I_II` for `[PHASE1, PHASE2]`, and `II_III` for
  `[PHASE2, PHASE3]`.
- NCI continues to reject `EARLY_PHASE1`; CTGov continues to accept it.

“The same token means the same thing” here means it reaches the same canonical
phase label/vector before source translation. It does not claim the providers'
search engines have identical overlap behavior. In particular, do not broaden
an existing scalar NCI query to include combined phases; single-phase request
behavior stays unchanged.

The deterministic round-trip proof is transform → shared normalization → exact
provider request construction. The receipted responses were not captured from
phase-filtered requests, so this ticket does not claim they prove the remote
provider would return the record for a newly constructed query. Do not use a
mock server that returns a fixture regardless of the request as proof of
selection.

## Errors and empty input

An unknown or malformed nonblank phase returns
`BioMcpError::InvalidArgument` during validation, before provider work. The
message names the supported canonical, Roman, numeric, and combined forms,
including `PHASE1/PHASE2`, `PHASE2/PHASE3`, `I_II`, and `II_III`.

Preserve existing blank behavior: an all-whitespace optional phase is treated
as absent by `normalized_phase_filter`. This ticket does not turn it into an
error.

## Advertised contract

The MCP trial-search schema currently uses a six-value enum and rejects emitted
values before Rust validation. Update its phase property so the schema accepts
the documented spellings in the table, including all exact emitted forms. Do
not attempt to enumerate every case variation; the schema documents canonical
and named alias spellings while Rust normalization remains case-insensitive.

Update the CLI phase help, `biomcp list` trial text, and phase tables/notes in
`docs/reference/quick-reference.md` and `docs/user-guide/trial.md`. The accepted
inputs, combined semantics, NCI translations, and NCI early-phase limitation
must agree across those surfaces. Update their existing contract tests rather
than adding an unowned documentation claim.

The first full `make lint` run exposed that `src/mcp/shell.rs` is held to an
exact 2,136-line legacy baseline by the Rust source-size ratchet. The MCP schema
change must not grow that file or update its allowlist baseline. Keep the
`typed_search_branch` call site compact and place the bounded trial-phase schema
and its focused unit test in a small `src/mcp/shell/` submodule (or an equally
bounded existing submodule), with only a line-neutral module/call-site change in
`shell.rs`. Do not hide the growth by weakening the ratchet.

The next full `make test` run exposed a second existing MCP constraint: the
stdio core contract caps serialized `tools/list` output at 16,000 bytes, and the
first expanded phase schema/help produced 16,002. Keep every required phase
spelling and meaning, but make the phase schema/help representation concise
enough to remain within the existing catalog budget. Do not raise the byte cap,
drop an accepted spelling, or weaken the contract test. The focused MCP test
must exercise the real serialized catalog size, not only inspect the phase
property in isolation.

The first two-byte remediation changed the MCP search catalog description from
“Search one biomedical entity...” to the equivalent “Search a biomedical
entity...”. Code review then caught that `manifest.json` carries the same public
description and its parity test requires exact equality. Apply that exact
wording change there too. This synchronization is authorized; no other plugin
manifest field or tool description may change.

## Done, observably

- Tests load the two named receipted records, derive the exact emitted strings
  through the real CTGov/NCI converters, and assert `PHASE1/PHASE2` and `III`
  remain byte-for-byte unchanged.
- Those derived outputs pass public trial-search validation for both sources.
  CTGov request construction yields the exact two-term `AND` expression for
  `PHASE1/PHASE2` and `AREA[Phase]PHASE3` for `III`; NCI construction yields
  `I_II` and `III`, respectively.
- Table-driven tests cover every retained scalar alias and both combined
  families, including `PHASE2/PHASE3`, `2/3`, `I_II`, and `II_III`.
- Malformed compounds fail atomically as `InvalidArgument` before request
  construction. Existing invalid-value coverage remains, and its accepted-list
  assertion includes the new forms.
- NCI still rejects `EARLY_PHASE1`; existing scalar provider request mappings
  and CTGov combined `AND` behavior remain pinned.
- The MCP schema admits the documented exact spellings and every emitted form;
  CLI/list/reference/user-guide surfaces advertise the same bounded contract.
- The schema test lives outside the oversized `src/mcp/shell.rs`, whose exact
  source-size baseline remains unchanged and passes the quality ratchet.
- The real stdio `tools/list` core contract remains at or below its existing
  16,000-byte limit with no budget increase and all required phase spellings.
- `manifest.json` and the typed MCP catalog retain exact public-description
  parity after the two-byte wording change.
- `make lint`, `make test`, and `make spec` pass.

## Boundary

Change shared phase normalization, the two source-specific phase request
mappings only as needed for the new canonical combined vector, their focused
tests, the named schema/help/docs surfaces, and the exact synchronized search
description in `manifest.json`. Tests may consume the two
already-receipted fixtures.

Do not add the CTGov transport capture to `fixture_key_contract.on_disk`.
Ticket 1126 intentionally limits that selector inventory to conversion
fixtures under `testdata/sources/clinicaltrials/` and
`testdata/sources/nci_cts/`; `_consumed_trial_files` does not discover the
separate `testdata/sources/ctgov/` transport-capture directory. The CTGov file
is already classified and receipted in the manifest, which is the provenance
needed here. Declaring it in the narrower selector inventory produces a false
`declared trial fixture is not consumed` failure. The NCI fixture is already a
declared `nci_cts` conversion fixture. Do not change
`testdata/sources/capture-receipts.json`, capture bytes, receipts, attestors,
exceptions, or checker behavior for this ticket.

Do not change `Trial.phase` or `TrialSearchResult.phase`, provider response
conversion/rendering, scalar NCI query expansion, status normalization, or any
other trial filter. Do not make live provider requests or alter capture bytes.

## History

Proposed 2026-09-02 for the CTGov joined value. Amended 2026-09-03 to absorb
the equivalent NCI Roman-numeral defect. Independent evidence and design review
then replaced an unreceipted fixture, reconciled the scalar public output with
the internal vector, bounded the emitted vocabulary, distinguished canonical
normalization from provider overlap semantics, added the MCP schema boundary,
and made the deterministic request-construction proof explicit. Implementation
then exposed that a demanded `ctgov/` selector declaration contradicted ticket
1126's narrower conversion-fixture inventory; the boundary above records the
correct separation between receipted transport evidence and that selector
inventory.

## Completed 2026-09-04

Implemented the accepted scalar and combined phase vocabulary without changing
the public scalar output bytes. Receipt-backed tests now prove the deterministic
emitted-value-to-normalization-to-request path for CTGov and NCI; they do not
claim that the existing captures were selected by remote phase filters. The MCP
schema, CLI/list help, and documentation agree, while `src/mcp/shell.rs` remains
at its 2,136-line baseline and the serialized `tools/list` response remains
within the existing 16,000-byte cap. The exact catalog wording adjustment was
synchronized to `manifest.json` to preserve the public manifest parity
contract.

Independent design review accepted the ticket after its evidence, fixture
inventory, source-size, catalog-budget, and manifest-parity amendments. A fresh
code review accepted the final implementation after the manifest parity finding
was remediated.

Final gates passed on the reviewed tree: `make lint`; `make test` with 3,064
Rust tests passed (30 skipped) and 877 Python tests passed (3 skipped), including
the strict documentation build; and `make spec`, including its static lane.
