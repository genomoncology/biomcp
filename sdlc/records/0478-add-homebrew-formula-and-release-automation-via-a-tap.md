---
base: 23c9d8764116aba343e8354fc3617817ec380300
head: 7aa7b8948d35950e3e2ebadbf9ef6b1f53682d8b
---
A `brew install` path serves the large Mac-native developer audience that does not use `uv`/`pip`. BioMCP's single self-contained binary is ideal for a Homebrew formula. The ongoing cost is a per-release formula bump, which should be automated so it is not manual toil.

Imported from March ticket 478. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/478-add-homebrew-formula-and-release-automation-via-a-tap
