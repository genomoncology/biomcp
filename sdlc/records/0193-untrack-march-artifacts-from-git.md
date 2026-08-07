---
base: e0b9ed7e27315e052fc6561e9f2859cfb5c6fba7
head: ac39151341f626f07589618a26f547fb8ee64041
---
Two `.march/` runtime artifacts are tracked in git by accident: `.march/code-review-log.md` and `.march/publish-report.md`. Both are per-run step outputs that leak review artifacts from one branch into another branch's diff. They were committed before `.march/` was added to `.gitignore`.

Imported from March ticket 193. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/193-untrack-march-artifacts-from-git
