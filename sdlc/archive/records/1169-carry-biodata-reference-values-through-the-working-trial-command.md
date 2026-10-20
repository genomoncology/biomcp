---
flow: build
priority: 1
status: complete
---
# Carry BioData reference values through the working trial command

## Outcome

The working BioMCP clinical-trial retrieval path stores each reference in BioData's `ClinicalTrialReference` instead of storing its own duplicate PMID, citation, and type fields. CLI Markdown and JSON retain useful reference information and existing section behavior. This is the first incremental library adoption in the product integration branch.

## Current facts

At BioMCP `f68d88328e77e4e1d11a2e868d88acc04311470c`, `src/entities/trial/mod.rs` defines `TrialReference` with three public fields. `src/transform/trial.rs` constructs it after trimming optional source strings and dropping rows without a usable citation. The renderer serializes these fields for its template. Two renderer tests construct the old struct directly. This baseline includes the twenty-six recently completed BioMCP fixes and passes the repository gates and hosted CI.

BioData revision `4f912d35a0f3fbff6994f1769d7601d7d0367aa1` provides public `ClinicalTrialReference` and `ExtensibleCode` values with validating constructors and borrowing getters. It also records the approved two-provider delivery contract. Its complete trial projection requires unrelated modules and cannot replace the narrow references path safely in this ticket. This ticket moves shared-value ownership only. The existing CTGov decoder and product trimming/filtering remain in place.

Ian authorized incremental BioData adoption in a dedicated BioMCP worktree on 2026-09-05. Reuse the existing application, preserve useful functionality, and allow improved answers. Exact historical output parity is not the goal. No cleanup task is a prerequisite. BioData remains an exact Git dependency through internal BioMCP 1.0 use. Public packaging waits for the public BioData release milestone.

## Scope and decisions

Add a normal Cargo Git dependency on `https://github.com/genomoncology/biodata` pinned to the full revision above and update the lockfile without unrelated dependency upgrades. Do not commit a local path dependency, vendor copy, or submodule.

Replace the product's stored reference fields with one private shared value. A small product wrapper may preserve the existing required `citation` and optional `pmid`/`reference_type` JSON shape. Any wire helper is temporary serialization state, not another authoritative runtime copy. Delete the wrapper before the checkpoint after arms and eligibility. The later reference path must render from the BioData document or use direct serialization. Keep the template unchanged if possible. Use the source-owned `clinicaltrials.gov` authority for CTGov reference type codes.

Preserve source normalization: null/missing/blank optional values become omitted product fields, a missing/null/blank citation is excluded as today, and valid references retain order and Unicode. Do not silently discard a row because a shared constructor failed. Propagate a sanitized typed product error if construction fails. Update callers explicitly if conversion becomes fallible.

The product never intentionally emits empty citations. Its reference wire decoder must reconstruct the shared invariant: reject an empty required citation with a deserialization error, and normalize empty optional fields to absence consistently with the source conversion. Test this deliberate internal wire tightening. Do not loosen BioData constructors to preserve malformed internal records.

Use existing CLI/fixture-server test infrastructure to prove the real product path for at least two IDs, populated/empty references, optional null/blank values, types and order, changed provider citation, and `references`/`all` section requests. Retain a non-reference trial command and 404 regression. Source records lacking sponsor/design/conditions must continue to serve valid references. Reuse existing admitted payloads and labelled synthetic test derivatives; do not capture live data or invent provenance.

The main agent owns BioData planning and ADR updates. This ticket changes only the BioMCP integration worktree. Rebase the existing reviewed work onto the named BioMCP baseline before final review. Do not dispatch factory work, migrate the full source adapter, alter HTTP, cache, retry, or request construction, change provider scope, change release versioning, or grow a second standalone product.

## Acceptance

1. Inspection and a focused type-level test prove one BioData value owns reference data. The old product field storage is gone, and no fallback representation remains.
2. Focused construction and serialization tests prove source normalization, required citation, omitted optional members, source type, Unicode and ordering. Constructor failures do not become silently missing rows.
3. The actual CLI against a local fixture server proves the stated retrieval cases and reflects changed reference content. Assertions check capabilities and information rather than a stored complete-output hash. Existing transport and source parsing tests remain intact.
4. Independent design and code reviews accept the work. Run focused red/green tests, then `make lint`, `make test`, and `make spec`. A live package contract records the extracted-package compilation deferral while BioData remains an exact Git-only dependency. That contract names the public release milestone and exact revision. Every package-content boundary remains active.
5. Record limitations honestly. Shared values are adopted. BioData's CTGov adapter and original-byte capture projection are not yet integrated.
6. Rebase onto BioMCP `f68d88328e77e4e1d11a2e868d88acc04311470c` and update the exact BioData pin to `4f912d35a0f3fbff6994f1769d7601d7d0367aa1`. A mechanical ancestry check must prove that the named BioMCP baseline is an ancestor of the final commit. The complete post-rebase BioMCP gates must pass. Existing regression tests then protect the twenty-six fixes without a second parity catalog. Do not restore ticket 1132 or any behavior that those fixes removed.

## Dependencies

The pinned BioData revision is committed and available. No library API addition is required. Work remains on the dedicated integration branch pending review for merge. Both factory channels remain paused. This ticket runs through the manual subagent SDLC.

## Review

- Prior design review: accepted against the earlier baseline. The product wrapper must require a usable citation on every construction path, including decoding or conversion from a shared reference, because BioData itself permits an absent citation.
- Prior code review: accepted against the earlier baseline after the serializer began borrowing through the shared getter to satisfy the unused-code lint. No suppression or behavior change was introduced.
- Refreshed design review: accepted against BioMCP `f68d88328e77e4e1d11a2e868d88acc04311470c` and BioData `4f912d35a0f3fbff6994f1769d7601d7d0367aa1`.
- Current code review: accepted after the rebase and package-boundary remediation. The reviewer found no blocking issues and independently verified ownership, serialization, package policy, baseline ancestry, the exact dependency pin, focused tests, and the complete recorded gate result.
- Packaging remediation: Ian deferred public BioData publication until BioMCP 1.0 is complete and used internally. A live test now enforces the exact Git-only dependency state and names that milestone. The Rust wrapper and real-process tests moved into existing owner files. `cargo package --list --allow-dirty --locked --offline` reports exactly 1,300 files. Package-content and private-root boundaries remain active.

## Verified progress

One private BioData reference value now owns product reference data. Focused red tests observed 1,302 package files and the stale BioData revision before remediation. The focused green lane passed three Rust reference tests and thirteen package and CTGov process tests. The real process tests cover two recorded trial IDs, populated and empty references, optional null and blank fields, Unicode, reference order, changed synthetic citation content, `references` and `all`, a non-reference section, and a 404.

The branch now rebases cleanly onto BioMCP `f68d88328e77e4e1d11a2e868d88acc04311470c`. `git merge-base --is-ancestor` exits zero. Cargo and its lockfile pin BioData `4f912d35a0f3fbff6994f1769d7601d7d0367aa1`. The package contains exactly 1,300 files. The model owner remains below the 1,000-line Rust threshold at 909 lines.

Post-rebase `make lint` passes. Post-rebase `make test` passes 3,152 Rust tests with 30 skipped, 897 Python tests with three skipped, and the strict documentation build. Post-rebase `make spec` passes every routine and static suite. `git diff --check` passes. Independent code review accepted the result. Merge this area independently. Do not hold it for later clinical-trial areas.
