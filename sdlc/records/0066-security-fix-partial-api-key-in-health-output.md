---
base: 041cb62196cdaf1c35581f861e14699ae3e4e8fb
head: 20a4cb19fb6f06242140550ebd5b1fc42a9177c1
---
`biomcp health` and `biomcp health --json` embed the first 3 characters of every configured API key in the `status` column/field:

Imported from March ticket 066. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/066-security-fix-partial-api-key-in-health-output
