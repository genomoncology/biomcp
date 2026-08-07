---
flow: build
priority: 5
---
# Report rejected DisGeNET credentials truthfully

`biomcp --json health --apis-only` can see a configured `DISGENET_API_KEY` and report that DisGeNET rejected it with HTTP 403, but `biomcp --json get gene BRAF disgenet` maps the same response to `api_key_required` and tells the operator to set the already-set variable. This makes a real provider rejection look like local setup omission. Fix the error contract without removing DisGeNET, exposing credentials, or making routine gates depend on a live account.

Completed under March on 2026-07-13, as March ticket 508. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/508-report-rejected-disgenet-credentials-truthfully
