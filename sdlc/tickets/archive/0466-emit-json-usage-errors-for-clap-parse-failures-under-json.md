---
flow: build
priority: 7
---
# Emit JSON usage errors for clap parse failures under --json

When `--json` is present, scripts reasonably expect parseable JSON even for usage mistakes. Known issue 441 remains: clap parse errors such as missing required arguments or unknown subcommands exit before BioMCP error rendering, leaving stdout empty and printing plain clap text to stderr.

Completed under March on 2026-06-30, as March ticket 466. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/466-emit-json-usage-errors-for-clap-parse-failures-under-json
