---
base: dab68f6711d3553168267faf96e176c45f18f30a
head: 9a0e144773f8670393c537b0152944c3c09fdac9
---
Make -j emit a JSON error object on stdout for every error class (not just some); today not_found/InvalidArgument go to stderr with empty stdout, breaking jq piping. Low severity (MCP unaffected).

Imported from March ticket 441. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/441-j-emit-json-error-object-on-stdout-for-every-error-class
