---
base: 28034bb1c4f70fc18622e13b7aab10f3955965bb
head: 4a67d4ad80f14391a65eda3fdac08311a0fa9cdf
---
WHO publishes 284 prequalified vaccines covering the diseases that determine drug access for hundreds of millions of patients (HIV, TB, malaria, hepatitis, measles, HPV, COVID-19, yellow fever, polio). Spike 231 proved these can't load through the MyChem/INN pipeline (57% match), but with the CVX identity layer (ticket 233), vaccine names can map through CVX codes instead.

Imported from March ticket 234. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/234-load-who-prequalified-vaccines-via-cvx-identity-bridge
