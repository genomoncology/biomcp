---
priority: 2
---

# 2259 — Check replay recordings against live providers before release

Status: OPEN.

Milestone: 0.9.4

## Outcome

Before every release, one opt-in live check replays each recorded provider request in `testdata/replay` against the live provider and reports every recording whose live reply now differs in status, content type or the response fields BioMCP reads. A drifted recording is re-recorded or its test is fixed before the tag, so a provider change cannot hide behind green replay tests.

## Evidence

Routine tests and specs run offline against recorded replies. `make release-live-smoke` runs a handful of live commands and the live verify spec lane, but nothing compares the recordings themselves with today's providers, nothing says when recordings are refreshed, and the release checklist names no live step.

## Keeps

Opt-in only, never in routine gates. Pace every provider and respect each provider's terms and the disabled-source groups; no licensed source is called without its key. Compare the fields BioMCP parses, not whole bodies, so timestamps and ordering noise do not count as drift. Record no credentials.

## Proof

The check runs once on the 0.9.4 candidate and its report is retained in a numbered experiment folder. A recording deliberately edited to a stale shape is reported as drifted. The release checklist gains the step: run the drift check, then re-record or fix every drifted recording before the go request. Document when and how to re-record in the testing docs.
