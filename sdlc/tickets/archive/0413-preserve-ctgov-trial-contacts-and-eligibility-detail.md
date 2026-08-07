---
flow: build
priority: 10
---
# Preserve CTGov trial contacts and eligibility detail

Survey issue 4 found that the ClinicalTrials.gov detail path drops action-critical fields before rendering: module-level central contacts, contact email, and structured sex eligibility. Rare-disease trial workflows need these fields to answer practical site/contact/eligibility questions without direct CTGov API inspection.

Completed under March on 2026-06-13, as March ticket 413. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/413-preserve-ctgov-trial-contacts-and-eligibility-detail
