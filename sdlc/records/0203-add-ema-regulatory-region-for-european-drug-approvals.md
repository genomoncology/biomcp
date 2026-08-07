---
base: 37f69d402ec04ab09655c32578eba89aabbe845d
head: 1455f2ffab68a1d3792b8d201dda9b2ba9a45213
---
BioMCP's drug regulatory data comes exclusively from FDA/openFDA. Questions about European drug approvals, EMA authorization dates, and EU-licensed products return no data. BioASQ evaluation (research 009) found 2 tasks that directly failed because of this gap: dupilumab EMA approval date and European influenza vaccine brand listing. A third task (thalidomide indications) was partially affected because POEMS is approved in Japan/Europe but not FDA.

Imported from March ticket 203. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/203-add-ema-regulatory-region-for-european-drug-approvals
