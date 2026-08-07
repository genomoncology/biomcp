---
base: 4bdc61a37f8ec482933ba0bb63e2a40b2f6bc769
head: a573243f455107e13e9e9b5a573cc3081bb186a6
---
`spec/17-cross-entity-pivots.md` line 23's mustmatch assertion contains backticks that bash evaluates as command substitution, producing `bash: line 8: get: command not found` and masking the real assertion. The test has been broken for weeks; pre-existing on main, not introduced by ticket 248.

Imported from March ticket 276. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/276-fix-spec17-bash-backtick-quoting-so-mustmatch-assertion-runs
