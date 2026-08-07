---
base: a7bd14c8d7c4be5cec02b259f912615850d87728
head: 9b0e63be59d8c25b85385d2809c0c52c1c1f7022
---
`biomcp search adverse-event -d <drug> --count total` is accepted but returns empty results — verified: `--count total` yields `"buckets": []` while a real field like `--count patient.reaction.reactionmeddrapt` returns data. openFDA has **no `total` count field**; the report total lives in `meta.results.total`, which is a different concept from a faceted `&count=<field>`.

Imported from March ticket 424. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/424-adverse-event-count-reject-the-fake-total-field-instead-of-silently-returning-empty
