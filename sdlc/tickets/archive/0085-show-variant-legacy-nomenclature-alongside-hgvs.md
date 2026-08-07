---
flow: build
priority: 7
---
# Show variant legacy nomenclature alongside HGVS

`search variant -g PLN` shows HGVS notation only (p.L39X, p.R25C). BioASQ gold uses legacy notation (PLN L39stop, PLN -42 C>G, Arg(9) to Cys). Both refer to the same variants. Agents can't match the formats.

Completed under March on 2026-03-29, as March ticket 085. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/085-show-variant-legacy-nomenclature-alongside-hgvs
