---
base: ad4c4f962558ba60c204f4235734561ea8110c60
head: 58ca87769c0ef62c07828899fdeb7ac6fec547c7
---
Repair the Option<u64> normalized_id mismatch that makes every PubTator export response undecodable, and replace the over-minimized capture that hid it

Imported from March ticket 632. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/632-accept-string-and-numeric-pubtator-normalized-id-and-anchor-the-export-fixture-to-a-real-capture
