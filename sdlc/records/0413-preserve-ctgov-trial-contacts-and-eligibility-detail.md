---
base: 79431c5fc48593613c4a5c03014b6b89223b58a6
head: 882cd71224b0ccc57b5ef276d8c500254e0b979b
---
Survey issue 4 found that the ClinicalTrials.gov detail path drops action-critical fields before rendering: module-level central contacts, contact email, and structured sex eligibility. Rare-disease trial workflows need these fields to answer practical site/contact/eligibility questions without direct CTGov API inspection.

Imported from March ticket 413. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/413-preserve-ctgov-trial-contacts-and-eligibility-detail
