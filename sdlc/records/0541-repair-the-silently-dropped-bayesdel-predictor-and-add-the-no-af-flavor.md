---
base: e56630be36ba69644eb857a045aa35c6cc3c0f31
head: 04acdc11511026396a70d6e2ca5470cb0087c370
---
BioMCP requests a BayesDel score from MyVariant, builds a prediction entry for it, and ships nothing. The field path is stale, so the source returns no value, `push_prediction` skips the tool because both score and prediction are `None`, and the omission is silent.

Imported from March ticket 541. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/541-repair-the-silently-dropped-bayesdel-predictor-and-add-the-no-af-flavor
