---
flow: build
priority: 5
---
# Lint: spec bash blocks without mustmatch pipe are silently uncollected

The mustmatch pytest plugin collects bash code blocks only when the block contains a `| mustmatch` pipe. Sections that use `jq -e ... > /dev/null` assertions alone are silently skipped — they show as neither pass, fail, nor skip. Ticket 195 discovered nine new "Search JSON Next Commands" sections that were silently uncollected; spec health metrics were overstated until the sections were repaired in-ticket. A lint that flags `## ` sections with bash blocks but no `| mustmatch` pipe would catch the whole regression class at spec-write time.

Completed under March on 2026-04-16, as March ticket 221. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/221-lint-spec-bash-blocks-without-mustmatch-pipe-are-silently-uncollected
