---
flow: build
priority: 6
---
# Suppress retry middleware warnings from stderr

When BioMCP hits rate limits on upstream APIs (notably Semantic Scholar), the `reqwest-retry` middleware emits WARN-level log lines to stderr like:

Completed under March on 2026-03-18, as March ticket 013. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/013-suppress-retry-warnings-stderr

The landed commit range could not be recovered from git, so no
record accompanies this entry. That is a known gap for the
earliest tickets, not a sign the work is missing.
