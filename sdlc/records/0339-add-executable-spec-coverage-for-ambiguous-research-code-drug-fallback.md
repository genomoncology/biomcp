---
base: cf5cc73e152ead99d80fcf07fdc2ca80e0d416db
head: 79b40c26326071dd06ed3cacb8e3f6c8ccd313a5
---
Ticket 310 shipped the canonical `MK-3475 -> pembrolizumab` research-code rescue with a contract assertion. The acceptance criterion that sparse research-code lookups with non-unique discovery signal must fall back to the existing alias-guidance surface (rather than rendering a misleading sparse card) has no executable-spec coverage. Build verified the runtime by inspection but cannot pin the contract.

Imported from March ticket 339. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/339-add-executable-spec-coverage-for-ambiguous-research-code-drug-fallback
