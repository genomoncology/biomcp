---
flow: build
priority: 5
---
# Preserve CTGov intervention aliases in trial next-command suggestions

BioMCP trial and article surfaces can suggest non-executable drug follow-ups when a ClinicalTrials.gov intervention is an investigational code rather than a resolvable drug identity. Example: `biomcp get trial NCT02136914 --json` currently suggests `biomcp get drug ADS-5102`, but `biomcp get drug ADS-5102` fails even though the CTGov record itself provides `otherNames: ["amantadine HCl extended release"]` and the summary says ADS-5102 is an investigational formulation of amantadine. Preserve source-provided trial intervention aliases and make next-command generation prefer safe search or resolvable alias follow-ups over brittle `get drug <code>` suggestions.

Completed under March on 2026-04-29, as March ticket 342. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/342-preserve-ctgov-intervention-aliases-in-trial-next-command-suggestions
