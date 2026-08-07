---
base: a935ad827a65b83890210c7bbd7cb96cfa14ccf4
head: 631b505e34c724fe0ec902e96c130bee8d594d32
---
ProviderCaptureStore validates directory components with symlink_metadata then writes by path, so a local attacker can swap a component between the check and the write.

Imported from March ticket 647. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/647-close-the-provider-capture-directory-symlink-toctou-race
