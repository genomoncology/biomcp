---
base: 9558eb3c4a509cdcd80a0f1fbb69983437e887c4
head: e0771e884a9ee46e652bdea89600a29bfa927e04
---
`biomcp suggest "What is the mechanism of resistance to imatinib?"` returns starter commands anchored on `"mechanism of"` instead of `"imatinib"`:

Imported from March ticket 291. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/291-fix-biomcp-suggest-resistance-to-drug-anchor-extraction-and-add-regression-test
