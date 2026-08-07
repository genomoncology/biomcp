---
base: 7f404c3109c20289b17e86f050e8dcebb40fd26c
head: fe1e095fe2a8520353baaa674d1b728c1033c93f
---
`search drug --indication "leukemia" --mechanism "purine"` doesn't consistently find all purine analog drugs. Agent searching for T-PLL purine metabolism drugs found pentostatin but missed deoxycoformycin (same drug, different name) and nelarabine. Gold expects: deoxycoformycin, pentostatin, nelarabine.

Imported from March ticket 086. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/086-audit-drug-mechanism-labels-for-purine-analog-completeness
