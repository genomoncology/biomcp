---
base: 090bad3b0363170730e68d1b13896eb6e3e1431d
head: d085b4bae94a24ba3d405bbbae066f9f6231bc01
---
biomcp emits suggested commands as shell strings throughout the rendered output (`See also:` blocks, `More:` follow-ups, error recovery hints). Some of these strings include drug/disease names with spaces, parens, or quotes. A copy-paste workflow can break when shell-active characters land unquoted. This is a cross-cutting hygiene issue with security-flavored implications (command injection if an upstream label contained `;` or backticks).

Imported from March ticket 313. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/313-unify-shell-quoting-across-emitted-suggested-commands
