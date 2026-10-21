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
