---
flow: build
priority: 7
---
# Add biomcp discover command with OLS4, UMLS, and MedlinePlus

BioMCP currently requires users to know the entity type upfront — `get gene`, `search drug`, `search trial`. A patient typing "chest pain", a researcher typing "Keytruda", or an agent trying "ERBB1" all hit friction because there's no concept resolution step. Experiments 035 and 036 validated a discovery layer using three APIs: OLS4 resolved 9/10 test queries with no auth, UMLS added clinical crosswalks (ICD-10, SNOMED, RxNorm) on 10/10 queries, and MedlinePlus provided useful plain-language context for disease/symptom queries.

Completed under March on 2026-03-20, as March ticket 028. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/028-discover-command

The landed commit range could not be recovered from git, so no
record accompanies this entry. The work products above are the
evidence that survives; the absence of a record is a gap in what
git can still prove, not a sign the work is missing.
