---
base: 27fedd2b467fdcc20fb15b1e208dba5cd1b504c8
head: 77e95e588c36414d4811924889274b892f6ee7c3
---
`make test-contracts` fails because `.march/code-log.md` is tracked in git in this branch. The repo cleanup contract (`tests/test_directory_submission_contract.py`) asserts that no `.march/`, `.claude/`, or `.agents/` paths are tracked. Additionally, `spec/13-study.md` uses `python` instead of `python3` in a bash block, which fails on systems without a `python` symlink.

Imported from March ticket 079. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/079-repo-cleanup-remove-tracked-march-artifacts-restore-test-contracts
