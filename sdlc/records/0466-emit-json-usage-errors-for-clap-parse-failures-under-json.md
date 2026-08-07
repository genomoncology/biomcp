---
base: 73913f0a9fabd8eee1288ce93dc0519c29b48b47
head: 1e96950f17e2a2d40295ca9b858163f22bc92380
---
When `--json` is present, scripts reasonably expect parseable JSON even for usage mistakes. Known issue 441 remains: clap parse errors such as missing required arguments or unknown subcommands exit before BioMCP error rendering, leaving stdout empty and printing plain clap text to stderr.

Imported from March ticket 466. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/466-emit-json-usage-errors-for-clap-parse-failures-under-json
