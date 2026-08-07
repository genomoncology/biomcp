---
flow: build
priority: 6
---
# Improve error messages for study download 403 and DisGeNET 403

Two error paths surface raw upstream error text without BioMCP guidance. `study download` with an invalid study ID returns a raw AWS S3 XML 403 error with no pointer to `--list`. DisGeNET 403 echoes the upstream "Unauthorized" message without mentioning `DISGENET_API_KEY`. OncoKB already handles this pattern correctly (names the env var and shows the `export` command), so the expected behavior is established.

Completed under March on 2026-04-15, as March ticket 212. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/212-improve-error-messages-for-study-download-403-and-disgenet-403
