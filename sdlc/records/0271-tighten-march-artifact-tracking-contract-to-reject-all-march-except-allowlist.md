---
base: 9829969daacfe3c84d3a8596ccf9cc598e630a3c
head: 2fcee1bdcffb10e4964dacc97eb634419c11f439
---
`.march/verify-log.md` was accidentally committed in ticket 235 and `.march/blueprint.md` is currently tracked from ticket 246's flow, both violating `.gitignore`. The existing cleanup contract (`test_repo_cleanup_removes_local_artifacts_and_deleted_dirs_from_git`) caught them, but caught them case-by-case. The durable fix is to tighten the contract to reject ANY `.march/*` path except a small explicit allowlist, and to add a pre-commit hook that refuses to stage non-allowlisted paths.

Imported from March ticket 271. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/271-tighten-march-artifact-tracking-contract-to-reject-all-march-except-allowlist
