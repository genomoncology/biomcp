---
base: be4410e66ac86fa231d83acbd8a8586b55831cb8
head: 50a410e9e9e9c58d6fb220977d56583a74e3f1c7
---
BioMCP cannot directly answer a standard literature task: find a clinician-researcher, establish which identity is intended, retrieve their publications, and summarize collaborators or indexed topics. The reported workflow fell back to roughly ten custom E-utilities steps. Full article authors and an honest author filter remove the immediate false-negative and false-positive bugs, but a durable author entity still needs identity resolution across name variants, affiliations, ORCID, Semantic Scholar, and PubMed. Building it as name search alone would confidently merge different people.

Imported from March ticket 516. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/516-design-an-evidence-backed-author-identity-and-publication-surface
