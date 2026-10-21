# 2340 — Replace Windows cache permissions exactly

Status: OPEN.

Milestone: 0.9.4

## Outcome

On Windows, first-encounter repair of BioMCP's saved-tree and cache directories leaves only the current user's access, as database repair already does. The repair replaces the protected DACL exactly, so a pre-existing explicit grant to Everyone or another user does not survive. Directories pass the same protection to new children. Share one permission policy between database and directory repair.

- Changes: `src/cache/private.rs`, `tests/managed_state_permissions.rs`, `.github/workflows/ci.yml`, `tests/test_ci_workflow_contract.py`, `CHANGELOG.md`.

- Proof: a native Windows test seeds a directory with an explicit Everyone grant and an inherited grant, runs repair, and shows only the current user's access, with inheritance to new files; the existing database repair test still passes; the Windows CI lane runs both.

## Evidence

- Coordinator restatement: Replace existing managed-tree and directory DACLs with the same exact protected current-user policy used by the database, including child inheritance.
- Reproduction and planned proof: Current main 9ce416c uses icacls /inheritance:r /grant:r USER. Windows command semantics preserve pre-existing explicit grants to other accounts; this is a source/semantic counterexample, not an executed native reproduction. Native outside-in controls will seed explicit Everyone plus inherited access, invoke real cache stats/cache clear repair, and check exact current SID access and new-child inheritance.
- Planned files and scope: src/cache/private.rs, tests/managed_state_permissions.rs, .github/workflows/ci.yml and CHANGELOG.md. The existing feature link already covers the integration test.

- On 2026-10-21, a fresh independent release review of integration 3c8aee87a found that src/cache/private.rs:443-450 repairs directories with `icacls /inheritance:r /grant:r USER`. That removes inherited grants but keeps pre-existing explicit grants, such as Everyone, which breaks private first-encounter repair. Database repair already replaces the protected DACL exactly. The finding comes from the code and Windows command semantics, without a native reproduction.

- Final implementation shares exact protected current-token-user DACL construction between database files and managed tree entries. Directory ACEs inherit to files and subdirectories. Native handles refuse reparse points and preserve hardlink validation; the weaker icacls/path-memo path is removed. The outside-in native control repairs both the cache root and recursive session tree through cache stats, then checks existing explicit/inherited broad grants are gone and new children inherit user-only access. The earlier cache-clear proposal was not the retained-object proof.
- Direct M5 make landing-gate job j2734 passed exit zero at 648ad99d1ee74772a0b0ca8c512e5b3f9d9aa289. Its complete raw log is retained on Yellow under data/biomcp/ticket-evidence/2340/j2734.log, SHA256 e0cb61272dbfdbc16fd320f558f6d8b4010a80fa70de219cea5a4b68d812b6b8. Earlier unplaced/started cancellations remain non-passes; no completed routine was duplicated.
- Actual native Windows run 39653152731 failed cache initialization with access denied before the permission regression. Its receipt remains failed; full log SHA256 bab6e2bd3a955b60d1414801c5cf4fd4c554f662a709f88870768eb7cf35e50d. The reviewed repair adds READ_CONTROL solely to the cfg(windows) tree-repair handle. Unix-compiled inputs and exact ACL policy are unchanged, so the passing M5 routine retains its actual source binding.
- Corrected native workflow_dispatch run https://github.com/genomoncology/biomcp/actions/runs/39654530866 succeeded at 1296dfa9e07d4b382e25c561ec3d0474df2a4b08 with windows_only=true. The new exact directory/child-inheritance regression and retained database/hardlink regression passed, along with existing stdio, concurrency, lock and selected deadline controls. This actual run establishes the access-right repair; the earlier hypothesis is not substituted for execution. Complete failed and successful logs/metadata are retained separately on Yellow under data/biomcp/ticket-evidence/2340/biomcp-2340-native-RUN.{log,json}. Successful log SHA256 a15fcae25fe2df190be60ebf08661ef38049d7a21b2e6b0322a61ae5cc8e45f5; metadata SHA256 2d25f5ef87f87e6b47213ff4f086fed670dd506f7a4c97714c1a71ed8eddfede.
- Fresh reviewer accepts 1296dfa9e07d4b382e25c561ec3d0474df2a4b08 after independently verifying both actual proof boundaries. The coordinator independently read the routine terminal pass and the successful native run's exact source. This Evidence-only update changes no gate input. Land with the other accepted release-review corrections as one batch.
