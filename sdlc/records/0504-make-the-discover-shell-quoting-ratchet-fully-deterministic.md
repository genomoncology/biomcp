---
base: 046814b919eab1bbe16b897342e6d3a3627244cb
head: 95189e201b99c6de8a5da5c07bfe814a4fd786ca
---
The routine shell-quoting contract in `spec/surface/cli-contract-ratchet.md` currently runs `biomcp --json discover 'NM_000248.3:c.1799T>A'`. That command contacts public OLS4 with an 8-second timeout even though the assertion only checks that a generated next command quotes `>` safely. Ticket 501 verification observed the local formatting contract fail when OLS4 timed out, then pass on retry.

Imported from March ticket 504. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/504-make-the-discover-shell-quoting-ratchet-fully-deterministic
