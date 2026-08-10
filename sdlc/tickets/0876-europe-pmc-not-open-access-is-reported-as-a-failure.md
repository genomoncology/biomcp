---
flow: build
priority: 6
deps: ["0951"]
---
# Read Europe PMC's not-open-access answer as absent

## Done when

A Europe PMC HTTP 200 XML errorBean whose code/message explicitly says the
article is not open access becomes SourceAttempt::Absent. It emits no retry
warning and cannot turn an otherwise successful full-text/asset result into
sources unavailable.

Other XML errorBean responses, a malformed body, or a non-ZIP 200 without the
known permanent-absence meaning remain failures with a bounded safe reason.
Content type alone is not enough to classify absence.

## Proof required

- The real receipted response for PMID 30311380 passes through the production
  Europe PMC parser.
- Local fixtures cover known not-open-access, another errorBean, malformed
  XML, invalid ZIP, HTTP failure, and valid ZIP.
- Entity outcome tests prove absent versus failed and the resulting recovery
  suggestion.
- JSON/Markdown prove a permanent absence is quiet while a real failure is
  source-attributed.
- Logs use the safe public error representation, never raw transport Debug or
  credential-bearing URLs.

## Authorized test changes

Design commits may restate Europe PMC parser fixtures, article asset/fulltext
outcome tests, and renderer/error expectations for the corrected
absent-versus-failed state. Do not weaken valid ZIP or body-size checks.

The src line ceiling may rise by at most 100 lines.
