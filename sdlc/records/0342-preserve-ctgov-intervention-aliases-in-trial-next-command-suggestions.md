---
base: 72ad5deba8f737bb938f10600d9465878b0aa453
head: 882f5f1e2163abb74d4b14c36094e422c0f67425
---
BioMCP trial and article surfaces can suggest non-executable drug follow-ups when a ClinicalTrials.gov intervention is an investigational code rather than a resolvable drug identity. Example: `biomcp get trial NCT02136914 --json` currently suggests `biomcp get drug ADS-5102`, but `biomcp get drug ADS-5102` fails even though the CTGov record itself provides `otherNames: ["amantadine HCl extended release"]` and the summary says ADS-5102 is an investigational formulation of amantadine. Preserve source-provided trial intervention aliases and make next-command generation prefer safe search or resolvable alias follow-ups over brittle `get drug <code>` suggestions.

Imported from March ticket 342. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/342-preserve-ctgov-intervention-aliases-in-trial-next-command-suggestions
