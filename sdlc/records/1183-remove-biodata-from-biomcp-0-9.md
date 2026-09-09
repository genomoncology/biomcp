---
flow: build
priority: 10
deps: []
---
# Remove BioData from BioMCP 0.9

## Goal

BioMCP 0.9 builds, tests, packages, and serves its current clinical-trial CLI and MCP contracts without a BioData dependency, checkout, repository, release, credential, or runtime concept. BioMCP locally owns the narrow strict trial values, request plans, and provider adapters its product uses.

## Current evidence

This urgent release-boundary defect is present on `origin/main` at `a694f1dc68c548a1ab7113de20ea6b08772e6d14` (`0.9.0-dev.6`). `Cargo.toml` has a direct exact Git dependency on `https://github.com/genomoncology/biodata` at `cfafc69d27c9a2fc74909f21692a418a8b17db83` and a `[package.metadata.biodata-development]` block that defers extracted-package compilation. `Cargo.lock` records `biodata 0.0.11`; `cargo tree --locked -i biodata` reports the inverse dependency tree in this orientation:

```text
biodata 0.0.11 (private exact Git revision)
└── biomcp-cli 0.9.0-dev.6
```

The ordinary dependency direction is `biomcp-cli -> biodata`; BioData in turn brings `quick-xml 0.42.0` and also uses dependencies already present elsewhere in the graph.

The required completed graph is:

```text
biomcp-cli
├── local trial values and strict JSON codecs
├── local CTGov detail plan and response adapter
├── local NCI CTS detail plan and response adapter
└── existing public dependencies only

BioData: no node, edge, checkout, generated copy, or runtime handoff
```

The dependency is not isolated. Production imports occur in `src/entities/trial/design.rs`, `eligibility.rs`, `get.rs`, and `mod.rs`; `src/render/markdown/trial.rs`; and `src/sources/clinicaltrials.rs` and `nci_cts.rs`. BioData supplies arm/intervention identities and relationship validation, eligibility and reference values/codecs, request-aware section states, exact CTGov and NCI detail plans, strict bounded response parsing, identity/cardinality checks, and sanitized adapter error codes. Rust and Python tests construct or assert those types and handoffs. `tests/test_source_package_boundary.py` requires the Git revision and compile deferral instead of proving an extracted package compiles.

The exact current test/comment handoffs are `src/entities/trial/get/tests.rs`, `src/entities/trial/test_support.rs`, `src/mcp/shell/typed_get_tests.rs`, `src/render/markdown/root_tests.rs`, `src/render/markdown/trial/tests.rs`, `src/sources/clinicaltrials/tests/{construction,parsing}.rs`, `src/sources/nci_cts/tests/{construction,live,parsing}.rs`, `src/transform/trial/tests.rs`, `src/transform/trial/tests/ticket_1132.rs`, `tests/json_error_contract.rs`, `tests/test_nci_filter_transport.py`, and `tests/test_source_package_boundary.py`. Current public prose coupling is exactly `docs/user-guide/cli-reference.md` and `docs/user-guide/trial.md`; no current `spec/**` or `skills/**` file names BioData, but those shipped surfaces must remain protected from reintroduction.

History shows why a raw revert is wrong. `e3f108d0` established strict CTGov reference retrieval; `554fc137` replaced the lenient NCI `trials/{id}`/untyped reader with the current field-selected `trials` request and strict envelope, identity, and cardinality rules; `c988e43d` established identity-bearing arm assignments; `aa6a7fe6` changed stored reference values; `aafb52b7` established rich cross-provider eligibility and default-detail behavior; `ce1d4748` moved strict eligibility JSON to BioData; and `7e85db24`/`e8f02aa8` established structured reference JSON and current Markdown fallbacks. The pre-`554fc137` NCI reader and the retired scalar `reference_type` output do not meet today's product contract.

Commit `f68d8832` and other pre-adoption revisions are historical evidence only. Implementation starts forward from current main `a694f1dc`; it must not check out, reset to, merge, or mechanically revert an old tree or commit range. In particular, the current 1150 completion chain and later overlap in `Cargo.toml`, `docs/user-guide/cli-reference.md`, `src/mcp/shell.rs`, and `src/sources/mod.rs` must survive. Preserve `bincode`, Tokio's `test-util` feature, `sha1`, cache/source request and deadline coordination, and every non-BioData ticket and record. A dependency that happens to occur in BioData's graph is not removable when BioMCP still uses it directly; only `quick-xml 0.42.0` must leave `Cargo.lock` when metadata proves it became unreachable. Remove `serde_json`'s `raw_value` feature only if the completed local strict codecs do not use `RawValue` and all gates prove it unnecessary.

Current public Rust exposure is narrower than the CLI/MCP surface but is not zero-coupled: `BioMcpError` publicly re-exports `TrialDesignError`, whose relationship variant and accessor expose `biodata::ClinicalTrialArmRelationshipError`. Exact Rust source compatibility for that foreign type and zero dependency are mutually exclusive.

Current forward-looking coupling also remains in the two user-guide statements, `sdlc/planning/clinical-trial-capabilities.md`, stale active ticket copies 1180 and 1181, and ticket 1182. Completed `sdlc/records/**`, archived tickets, historical planning notes, and the fixture authorship/license receipt in `testdata/sources/capture-receipts.json` are historical evidence, not executable coupling.

## Required behavior

Implement a clean BioMCP-local boundary. Do not vendor, subtree, copy wholesale, rename, publish, or add a path/patch dependency on BioData. Do not require a sibling checkout, Git access, generated BioData source, environment variable, feature flag, CI secret, token, SSH key, or registry credential. Do not publish BioData or coordinate a BioData release. Historical commits and provider-backed BioMCP fixtures may be used as evidence; the implementation must remain the smallest product-owned code that satisfies BioMCP's contracts.

Locally own these values and invariants in the trial entity layer:

- source authority/code plus nullable display, vocabulary version, and recognized meaning, rejecting empty present values;
- positive portable arm, intervention, and eligibility-criterion identities in `1..=9_007_199_254_740_991`;
- interventions, arms, arm/intervention assignments, and typed duplicate/missing-endpoint relationship errors;
- request-aware `Present`, `Absent`, `NotRequested`, and `Unavailable` section states where the adapters require them;
- eligibility registry text, age range/bounds, source duration, sex codes, healthy-subject state, ordered criteria, classification, and the NCI `999 Years` source-stated-no-limit rule;
- references with exactly nullable `pmid`, `citation`, and `source_type`, where a populated source type has exactly the five code members.

Keep `TrialDesign` as the owning graph boundary. Replace its foreign relationship error with a local error that retains the current variant names, fields, equality/copy behavior, data-free `Display`, `Error::source`, and accessor semantics. Re-export the reachable local error beside `TrialDesignError`. This is the one accepted pre-1.0 Rust source change: callers naming or downcasting the foreign BioData type must migrate to BioMCP's local type. No successful CLI, MCP, JSON, Markdown, or shell contract may change.

Implement local strict value codecs. Eligibility keeps all five required members; references keep all three required members and all five required nested code members, including explicit nulls. Preserve missing/null/empty distinctions, Unicode, order, source values without storage-time trimming, duplicate-member rejection at the root and nested levels, unknown/missing/wrongly typed member rejection, portable identity bounds, duplicate criterion rejection, exact duration source quantity/unit/bound validation, and fixed data-free errors `invalid clinical trial eligibility` and `invalid clinical trial reference`. Markdown alone may trim for display and must retain the PMID-only behavior from `e8f02aa8` and all current safe fallbacks.

Implement only the local provider adapters BioMCP consumes. Do not recreate BioData capture reports, conversion reports, generic cross-product trial projections, legacy XML, or unrelated values.

The CTGov adapter must preserve the exact normalized `studies/{NCT ID}` path; sorted, deduplicated `fields` query; always-requested base/intervention fields; default-detail eligibility; explicit/all section composition; eligibility-triggered document fields; request-aware references, arms, and eligibility; exact returned-identity validation; typed arm occurrence assignments; and current reference and eligibility projection. It must continue to parse the same untouched response bytes used by the product's ordinary study model.

The NCI adapter must preserve `GET trials`, `size=1`, normalized `nct_id`, every ordered repeated `include` pair, the single `X-API-KEY` header, eligibility inclusion only when requested/defaulted, and `brief_summary`. It must preserve status-before-body handling, zero/one/multiple-row behavior, exact raw returned-identity validation, rejection of the old lenient shapes, required field meanings, exact whole portable accrual conversion, nested arm/intervention occurrence identities, stable numeric `display_order` eligibility ordering, classifications, and current product projection. The API key must never enter a credential-free plan, cache key, model, error, log, fixture, or output.

Retain finite defense in depth. The transport already caps ordinary provider bodies at 8 MiB; local strict decoding must also enforce inclusive maxima of 8 MiB input, depth 128, 4 Mi decoded Unicode scalars in one string, 1,000,000 elements in one collection, 16,384 members in one object including duplicates, and 1,000,000 normalized events. Check HTTP status before JSON validation. Preserve adapter codes `malformed_json`, `unsupported_json`, `json_resource_limit`, `invalid_projection`, and `identity_mismatch`, plus NCI `not_found` and `unexpected_row_count`. Only `json_resource_limit` maps to narrow-request recovery; other validation failures retain retry-source recovery. Errors and logs must not contain response bytes, rejected values, decoder paths, trial/contact/criterion sentinels, credentials, or local paths.

Remove the runtime phrase `BioData response validation failed` in favor of a provider-neutral, data-free validation message while preserving the structured BioMCP error code, source, recovery action, exit status, stdout-only JSON discipline, and no-partial-output behavior. This intentional human-message cleanup is required by zero runtime coupling; successful public output remains byte-compatible.

## Zero-coupling surfaces and ratchet

Remove the dependency, lockfile package/edge, compile-deferral metadata, imports, foreign type names, helper names, runtime messages, comments, and ownership assertions from these active surfaces:

- `Cargo.toml`, `Cargo.lock`, `Makefile`, `.github/**`, `scripts/**`, `tools/**`, and package/release configuration;
- production and test Rust under `src/**` and `tests/**/*.rs`;
- Python executable contracts under `tests/**/*.py`;
- public `README.md`, `docs/**`, `skills/**`, `spec/**`, examples, and schemas;
- current planning and active tickets under `sdlc/planning/clinical-trial-capabilities.md` and `sdlc/tickets/`.

Add a case-insensitive, default-deny static source/package ratchet that inspects every Git-tracked textual file and every textual member of the produced `.crate`. It fails on `biodata`, the BioData Git URL/revision, Cargo source/package entries, checkout/path/patch dependencies, deferred-package exceptions, or equivalent renamed/generated handoffs. Do not implement the scan as a list of active directories, file extensions, or selected production roots: an unclassified new tracked or packaged text file is scanned by default.

Historical exceptions are checked-in exact occurrences, not wildcard directory exclusions. Seed the exception inventory only from the already-present matching occurrences in these exact files: `sdlc/planning/notes/2026-09-03-splitting-the-nci-field-name-bundle.md`; the existing matching completed records 1111, 1116, 1121, 1122, 1126, 1136, 1141, 1150, 1152, 1154, 1157, 1166, 1168, 1169, 1170, 1171, 1175, 1176, 1177, 1178, 1180, and 1181 under `sdlc/records/`; the existing matching archived tickets 1110, 1118, 1119, 1133, 1134, 1135, 1137, and 1165; the archived 1182 produced here; the completed 1183 record produced here; and `testdata/sources/capture-receipts.json`. Each exception binds the exact path and existing occurrence text or content digest, so adding another occurrence to an allowed file, adding a new file under an allowed directory, renaming a file, or packaging an unexpected copy fails. Packaged text has no blanket historical exception; only an exact packaged member occurrence explicitly present in the inventory may pass.

Build dynamic temporary-repository and temporary-archive self-tests rather than testing only the real tree. Prove a forbidden manifest dependency, Rust import, test handoff, public-doc claim, active-ticket dependency, Git URL, mixed-case spelling, lockfile source, unknown extension, extensionless text file, new record, extra occurrence in an allowed record, renamed allowed file, generated source, and packaged-only textual member are rejected. Prove binary members are not decoded as text and each exact historical/provenance occurrence passes. The test fixture must not copy the production allow/deny result as its oracle. The ratchet runs in `make lint` through the existing quality-ratchet lane and in source-package tests.

Replace `tests/test_source_package_boundary.py`'s exact-revision/compile-deferral assertions with positive no-dependency and extracted-package verification. Run these exact network-denied commands from a clean focused tree:

```text
tools/run-offline -- cargo metadata --locked --offline --format-version 1
tools/run-offline -- cargo tree --locked --offline
tools/run-offline -- cargo package --locked --offline
```

The package command uses Cargo's normal verification; do not pass `--no-verify`. Inspect `target/package/biomcp-cli-*.crate`, unpack it into a fresh temporary directory, run the same default-deny textual-member ratchet, and run `tools/run-offline -- cargo check --locked --offline` in the extracted crate. `--allow-dirty` is permitted only in a focused test that intentionally exercises pre-commit package contents; it is not permitted for completion evidence. Metadata, tree, package, archive, and extracted compile output must contain no BioData package, URL, or source edge.

Update the two current user-guide ownership statements to describe BioMCP's local trial contract. Update the current clinical-trial capability plan. Remove the stale active 1180 and 1181 ticket copies because their completion records are already authoritative. Move 1182 to the archive with a concise superseded-by-1183 note; its still-relevant NCI public-boundary proof belongs in this ticket's acceptance. At completion move this ticket's durable result to `sdlc/records/1183-remove-biodata-from-biomcp-0-9.md`, leaving no BioData-named active ticket to defeat the active-surface ratchet. Do not edit or delete completed records, archived historical facts, the dated planning note, capture receipts, provider fixture bytes, or fixture authorship/license provenance.

Apply this disposition exactly:

| Item | Disposition |
| --- | --- |
| 1166 capability contract | Retain its completed record; rewrite the live `sdlc/planning/clinical-trial-capabilities.md` introduction for BioMCP-local ownership. |
| 1169, 1170, 1171, 1175, 1176, 1177, 1178, 1180, 1181 | Do not edit their completed records. The new 1183 completion record states that their BioData-dependent implementation decisions are superseded while their historical evidence remains true. |
| 1172 | It is absent; do not invent a ticket or record. |
| 1173, 1174, 1179 | Unrelated work; retain unchanged. |
| Active 1180 and 1181 copies | Remove the stale ticket files; retain the corresponding records unchanged. |
| 1182 | Move to `sdlc/tickets/archive/`, mark it superseded by 1183, and absorb its local NCI public-boundary proof here. |
| 1183 | On completion create its record with the supersession statement and remove the active copy under repository convention. |

All other non-BioData tickets, records, plans, and implementation changes remain untouched.

## Change isolation

Before implementation, prove `git merge-base --is-ancestor a694f1dc68c548a1ab7113de20ea6b08772e6d14 HEAD`. At review, inspect a focused `git diff --name-status a694f1dc68c548a1ab7113de20ea6b08772e6d14...HEAD` and path-specific diffs for `Cargo.toml`, `Cargo.lock`, `docs/user-guide/cli-reference.md`, `src/mcp/shell.rs`, and `src/sources/mod.rs`. The diff must show forward replacement from current main, no reverse application of `f68d8832` or the cited adoption commits, no loss from the 1150 ancestry, no unrelated dependency or feature deletion, and no edits to unrelated tickets 1173, 1174, or 1179. Tests and reviewers must verify the preserved cache/source coordination rather than accepting an ancestry check alone.

## Done, observably

- The completed dependency graph has no BioData node or edge, and a clean build needs no BioData repository, package, network fetch, secret, or credential.
- Exact CTGov construction tests prove the current path, deterministic field query, default eligibility, every single section, `all`, documents interaction, and omission of unrelated fields. Exact NCI construction tests prove path, ordered repeated includes, optional eligibility, one credential header, and credential-free plan data.
- Local parser tests retain every current positive and negative case for malformed/duplicate/unsupported JSON, identity mismatch, NCI row cardinality, old lenient shapes, strict required fields, arm ambiguity and relationships, section states, eligibility ordering/classification, structured references, and sanitized failures.
- Resource tests accept every limit exactly at its ceiling and reject the first byte, depth, character, element, member, and event over the ceiling with `json_resource_limit` and narrow recovery.
- Local value tests prove every relationship-error variant, endpoint/duplicate validation, portable identity bounds, eligibility/reference exact JSON, null/empty distinctions, Unicode/order preservation, duplicate nested members, invalid no-limit rules, and data-free errors.
- `tests/test_ctgov_trial_search_detail_reuse.py`, the NCI public-boundary proof superseding 1182, typed/raw MCP trial tests, `tests/json_error_contract.rs`, Markdown tests, schemas, examples, and mustmatch trial specs prove unchanged successful CLI/MCP output and no partial output on failure. Red tests fail when description, `display_order`, classification, malformed-as-absence, assignment, reference fallback, request field, or resource-limit behavior is bypassed.
- The zero-coupling ratchet covers the worktree and produced source package, has the named bypass tests, and allows only the historical/provenance paths above.
- Local Linux completion requires `make lint`, `make test`, `make spec`, `make full-feature-check`, and the complete `make release-gate` under the repository-pinned toolchain and Node 22 requirements, plus the exact offline metadata/tree/package/extracted-check commands above.
- The `windows-contracts` lane passes `cargo check --locked` and `cargo test --locked --test managed_state_permissions` on `windows-2022`. Read-only review of release configuration proves the `x86_64-pc-windows-msvc` build uses the same dependency graph; do not run candidate staging to satisfy this ticket.
- A clean/offline `cargo metadata`, `cargo tree`, package verification, and extracted-package compile all pass, and their output contains no forbidden dependency evidence.

Local completion and hosted confirmation are separate evidence. Finish code, tests, ratchets, planning disposition, the initial 1183 record, and removal of the active 1183 ticket in one reviewed implementation/pre-closure SHA, then push it. The canonical Linux gates, `full-features`, `windows-contracts`, and `repository-contracts` jobs must all be green for that exact SHA. After those results exist, a final closure commit may edit only `sdlc/records/1183-remove-biodata-from-biomcp-0-9.md` to name the reviewed implementation/pre-closure SHA and its green hosted job URLs/results.

The final closure SHA is the sole bounded exception to the exact-SHA recording rule because a commit cannot record its own future hosted results. Prove with its parent diff that the closure commit changes only that record and does not change behavior, dependencies, package membership, source or test code, fixtures, gates, ratchet code or exception inventory, or release configuration. Push the closure commit and observe the same required ordinary hosted checks for that closure SHA; report those subsequent results and URLs in the final handoff rather than attempting a self-referential record amendment. Any closure diff outside the one record cancels this exception and requires a new reviewed pre-closure SHA and exact-SHA hosted evidence.

## Boundaries

This ticket does not change trial search, provider URLs, public success JSON or Markdown, schemas, section names, source attribution, fixtures, capture receipts, release version, unrelated dependencies, or AlphaGenome behavior. Pushing the focused implementation and closure branches and running or observing their ordinary non-release hosted CI are authorized and required. No provider or live-smoke calls, release-workflow dispatch, candidate staging, signing, promotion, release publication, BioData publication, new CI credential, or other external coordination is authorized. The work must not rewrite history, import BioData source under another name, or broaden BioMCP into a general biomedical data library.

## Dependencies

None. The required provider fixtures, product contracts, historical implementation evidence, and current behavior tests are already in BioMCP. BioData access is neither a prerequisite nor permitted for implementation or gates.

## Review

- Design review: accepted before implementation.
- Initial implementation review: `6baa38452acaed1bf482f26de36fbdbd73e03036`
  received REQUEST CHANGES; its findings are remediated in this reviewed
  pre-closure branch.
- Hosted canonical Linux, `full-features`, `windows-contracts`, and
  `repository-contracts` checks: pending for the reviewed pre-closure SHA.
  This initial record does not claim hosted success.

## Initial completion status

BioMCP now owns the strict trial values, codecs, request plans, and provider
adapters it uses. The runtime graph, source package, and active source surfaces
have no BioData dependency or handoff. The completed records for 1169, 1170,
1171, 1175, 1176, 1177, 1178, 1180, and 1181 remain unedited historical
evidence, while their BioData-dependent implementation decisions are
superseded by 1183. Ticket 1182 is archived and its NCI public-boundary proofs
are absorbed here. The stale active copies of 1180 and 1181 are removed; 1172
remains absent; unrelated work 1173, 1174, and 1179 remains unchanged.

Local Linux evidence on the reviewed tree is green: `make lint`, `make test`
(3,327 Rust tests and 920 Python tests, with documented skips), `make spec`,
`make full-feature-check`, and the complete `make release-gate`. The package
contract reports exactly 1,300 source members and proves archive ratcheting plus
a fresh offline extracted compile. The network-denied metadata, tree, package,
and extracted-check commands also pass. The exact reviewed pre-closure SHA will
be supplied by the commit containing this record and implementation.

Hosted results remain explicitly pending. Only a later record-only closure
commit may add the reviewed SHA and green job URLs under the bounded closure
rule above.

## Hosted closure evidence protocol

This pre-closure commit makes the one-time checker and inventory amendment
needed to avoid a self-referential record digest. The record bytes before and
after the sentinels below are immutable. A later closure commit may replace
only the bytes between the sentinels, once, using the checker's fixed closed
grammar; the checker-provided closure-diff helper must prove that its parent
and child differ only inside this block. Pending evidence makes no hosted
success claim.

<!-- biomcp-1183-hosted-evidence-begin -->
status: pending
<!-- biomcp-1183-hosted-evidence-end -->
