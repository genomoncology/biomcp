---
base: bf548ceaa2dfa830763b76cbf46ecfe54c774f39
head: 6bb4fbc119abb1d04e4463bfa955c860c5d7207f
---
The mustmatch pytest plugin collects bash code blocks only when the block contains a `| mustmatch` pipe. Sections that use `jq -e ... > /dev/null` assertions alone are silently skipped — they show as neither pass, fail, nor skip. Ticket 195 discovered nine new "Search JSON Next Commands" sections that were silently uncollected; spec health metrics were overstated until the sections were repaired in-ticket. A lint that flags `## ` sections with bash blocks but no `| mustmatch` pipe would catch the whole regression class at spec-write time.

Imported from March ticket 221. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/221-lint-spec-bash-blocks-without-mustmatch-pipe-are-silently-uncollected
