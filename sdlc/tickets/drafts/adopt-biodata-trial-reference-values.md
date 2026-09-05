---
flow: build
priority: 1
---
# Carry BioData reference values through the working trial command

## Outcome

The working BioMCP clinical-trial retrieval path stores each reference in BioData's `ClinicalTrialReference` instead of storing its own duplicate PMID, citation, and type fields. CLI Markdown and JSON retain useful reference information and existing section behavior. This is the first incremental library adoption in the product integration branch.

## Current facts

At BioMCP `27dd20908e960d4a7e48173f24ddec43032b9614`, `src/entities/trial/mod.rs` defines `TrialReference` with three public fields. `src/transform/trial.rs` constructs it after trimming optional source strings and dropping rows without a usable citation. The renderer serializes these fields for its template. Two renderer tests construct the old struct directly.

BioData revision `59655960505bca2ea1e09aa0dc90d95ff98efef9` provides public `ClinicalTrialReference` and `ExtensibleCode` values with validating constructors and borrowing getters. Its complete trial projection requires unrelated modules and cannot replace the narrow references path safely in this ticket. This ticket moves shared-value ownership only. The existing CTGov decoder and product trimming/filtering remain in place.

Ian authorized incremental BioData adoption in a dedicated BioMCP worktree on 2026-09-05. Reuse the existing application, preserve useful functionality, and allow improved answers. Exact historical output parity is not the goal. No cleanup task is a prerequisite. Checkout execution can start without a public release; the packaging limitation below prevents claiming full acceptance.

## Scope and decisions

Add a normal Cargo Git dependency on `https://github.com/genomoncology/biodata` pinned to the full revision above and update the lockfile without unrelated dependency upgrades. Do not commit a local path dependency, vendor copy, or submodule.

Replace the product's stored reference fields with one private shared value. A small product wrapper may preserve the existing required `citation` and optional `pmid`/`reference_type` JSON shape. Any wire helper is temporary serialization state, not another authoritative runtime copy. Keep the template unchanged if possible. Use the source-owned `clinicaltrials.gov` authority for CTGov reference type codes.

Preserve source normalization: null/missing/blank optional values become omitted product fields, a missing/null/blank citation is excluded as today, and valid references retain order and Unicode. Do not silently discard a row because a shared constructor failed. Propagate a sanitized typed product error if construction fails. Update callers explicitly if conversion becomes fallible.

The product never intentionally emits empty citations. Its reference wire decoder must reconstruct the shared invariant: reject an empty required citation with a deserialization error, and normalize empty optional fields to absence consistently with the source conversion. Test this deliberate internal wire tightening. Do not loosen BioData constructors to preserve malformed internal records.

Use existing CLI/fixture-server test infrastructure to prove the real product path for at least two IDs, populated/empty references, optional null/blank values, types and order, changed provider citation, and `references`/`all` section requests. Retain a non-reference trial command and 404 regression. Source records lacking sponsor/design/conditions must continue to serve valid references. Reuse existing admitted payloads and labelled synthetic test derivatives; do not capture live data or invent provenance.

The main agent owns BioData planning/ADR updates. This ticket changes only the BioMCP integration worktree. Do not merge or push this branch into BioMCP main, dispatch factory work, migrate the full source adapter, alter HTTP/cache/retry/request construction, change provider scope, change release versioning, or grow a second standalone product.

## Acceptance

1. Inspection and a focused type-level test prove one BioData value owns reference data. The old product field storage is gone, and no fallback representation remains.
2. Focused construction and serialization tests prove source normalization, required citation, omitted optional members, source type, Unicode and ordering. Constructor failures do not become silently missing rows.
3. The actual CLI against a local fixture server proves the stated retrieval cases and reflects changed reference content. Assertions check capabilities and information rather than a stored complete-output hash. Existing transport and source parsing tests remain intact.
4. Independent design and code reviews accept the work. Run focused red/green tests, then `make lint`, `make test`, and `make spec`; report any independently verified baseline failures without weakening gates.
5. Record limitations honestly: shared values are adopted; BioData's CTGov adapter and original-byte capture projection are not yet integrated.

## Dependencies

The pinned BioData revision is committed and available. No library API addition is required. Work remains on the dedicated integration branch pending review for merge.

## Review

- Design review: accepted. The product wrapper must require a usable citation on every construction path, including decoding or conversion from a shared reference, because BioData itself permits an absent citation.
- Code review: accepted after the serializer began borrowing through the shared getter to satisfy the unused-code lint. No suppression or behavior change was introduced.
- Design review reopened: rejected the assumption that the pinned Git dependency alone can satisfy the existing distributable-package gate. Keep this draft incomplete and the branch provisional. Do not merge or waive either failed package check.

## Verified progress and blocker

One private BioData reference value now owns product reference data. Focused Rust tests and four actual CLI tests against local provider replies pass. Full `make lint` passes. Full `make test` reports 3,150 Rust tests passed and 30 skipped; its Python lane reports 894 passed, three skipped, and two failures. The failures are `test_cargo_source_package_keeps_the_runtime_boundary` (1,302 files exceed the unchanged 1,300 ceiling) and `test_verified_package_compiles_focused_identity_test_after_extraction` (Cargo rejects a Git dependency without a registry version).

An isolated baseline package listing at `27dd20908e960d4a7e48173f24ddec43032b9614` contains 1,300 files. The two added files account for the difference. A refreshed official registry query found no `biodata` package. Cargo removes the Git dependency source when preparing a registry package; adding a version alone cannot supply that missing release. This disproves a delivery assumption in the original design. No source-package check has been weakened.

The primary agent ran `make spec` separately after the test gate stopped; it passed. The combined lint/test/spec result is still not green because the two package failures remain.

Choose the reversible branch hold for now. A separately approved BioData release could satisfy the dependency requirement but creates a public artifact and requires Ian's authorization. A different distribution arrangement avoids that immediate release but changes the delivery contract and adds maintenance. Keep the tested implementation without merging while that decision is unresolved. File-count remediation also remains required; do not raise the ceiling or hide required source. Full acceptance remains incomplete.
