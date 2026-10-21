# Expanded 0.9.4 release admission

Follow the [release process](../../docs/reference/release-process.md). This record binds the expanded candidate's review and qualification to actual source revisions. It grants no release authorization. Ian alone tags and publishes.

## Independent project review

Three fresh independent reviewers covered runtime/contracts, security/state, and packaging/releases/documentation. Their lead reconciled architecture, tests, tickets and the findings below at clean published integration source `3c8aee87a69c5745412e46ef706b9f11c7c209cc` on 2026-10-21. The verdict was hold for four confirmed medium defects, with no critical finding. This verdict is not a pass for a later corrected source.

| Finding | Evidence and broken property | Authorized correction |
| --- | --- | --- |
| Windows managed-state privacy | `src/cache/private.rs` invokes `icacls /inheritance:r /grant:r USER`, preserving pre-existing explicit grants to other accounts. Saved downloads use this repair, contrary to private first-encounter repair. Database files already use exact protected-user DACL replacement. | 2340 shares exact replacement with proper directory inheritance and requires a native seeded-grant regression. |
| Broad variant pagination | Provider offset precedes a different local sort/truncation in `src/entities/variant/search/mod.rs`. Actual consecutive pages repeat the same row. Exact-identity search is separate. | 2341 establishes one stable ordering before slicing and tests page unions. |
| Unshipped guide promise | AGENTS promises `share/biomcp/docs` and `share/biomcp/skills`, but actual workflow archives and maturin wheels package the executable. Tests exercise an unused packaging helper. | PM m3211 chooses 2338: describe embedded skill commands and the documentation site, remove the dead packaging path, and protect the truthful public promise. |
| JSON smoke validation | `scripts/installed-wheel-smoke.sh` accepts an opening brace without parsing the complete document, contrary to the runbook's malformed-JSON rejection promise. | 2339 parses complete stdout as an object, checks malformed/trailing/array negatives and exercises installed skill render/install. |

The Windows finding is conditional on pre-existing explicit grants and follows from code plus [Microsoft's documented icacls semantics](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/icacls). No native Windows reproduction or real user-data inspection occurred during review. Native execution remains required for its correction.

The pagination reviewer ran one isolated authored loopback CLI probe through Gator job `2330/j2686` at the pinned source. Three limit-one pages at offsets zero, one and two each returned `chr1:g.1A>T`; one limit-three page returned IDs one, two and three. Upstream requests used from/size pairs 0/40, 1/40, 2/40 and 0/120. No live provider or key was used. Exit zero means the probe confirmed the defect. Yellow retains `data/biomcp/ticket-evidence/2330/review-pagination/{probe.py,command.txt,fixture.json,j2686.log,SHA256SUMS}`; log SHA256 `0ef129259206014541c0aafd27906a97347ad382af62afb28a6fb5a28895402a`. Unique probe scratch was removed; the shared qualification checkout remains available.

Two low inconsistencies need PM disposition: the workflow documentation claims long-suite coverage without an enforced designated selection, although no actual long-marked product cases currently exist; pm also reports fifteen unreadable legacy records outside the active queue. Neither establishes fifteen additional bugs or a currently omitted long case. The mechanical ticket review found no duplicate active canonical IDs, missing declared prerequisites or dependency cycles. The two explicit open tickets at the reviewed snapshot were sound and correctly ordered.

Sampled strengths include shared CLI/MCP execution, exact variant identity rejection, truthful article incompleteness, exact PMID/MED metadata recovery, outbound destination validation, MCP mutation restrictions, credential-safe cache publication, retained cache identity checks and migration locking. Rehearsal separates qualification from publication and checks native installed wheels and platform compatibility. The review did not execute exhaustive provider/clinical correctness, native Windows permissions, race stress or final installed artifacts. Apart from the single pagination probe it ran no duplicate full gate, live collection, credential access or publication. Preserve unaffected review scope and review the corrections before rebinding the final verdict.

## Qualification binding

The earlier release source and rehearsal remain historical evidence in [the prior admission](2258-release-admission.md). They do not qualify the expanded product source. The final direct main gate, fresh-clone offline Linux proof, authorized keyed five-record check and complete untagged rehearsal must name their actual new source. Final job receipts and publication limits will be recorded here after execution.
