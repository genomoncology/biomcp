---
base: c7420ba2953a8fe2216a073baa4b3c6381d1732b
head: 30b39fb7ecfd34cf9b99484164c8432ea0079cc7
---
Two error paths surface raw upstream error text without BioMCP guidance. `study download` with an invalid study ID returns a raw AWS S3 XML 403 error with no pointer to `--list`. DisGeNET 403 echoes the upstream "Unauthorized" message without mentioning `DISGENET_API_KEY`. OncoKB already handles this pattern correctly (names the env var and shows the `export` command), so the expected behavior is established.

Imported from March ticket 212. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/212-improve-error-messages-for-study-download-403-and-disgenet-403
