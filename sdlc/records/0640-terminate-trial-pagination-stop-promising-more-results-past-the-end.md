---
base: a8d93878f1d55128dc9897ba4b365da27fa92cf4
head: aebe124c2858f8ee7fa1c7bd910455d2b8854007
---
A trial search past the end of the result set returns zero rows but still reports has_more true with a next_page_token, so an agent paginating on has_more never terminates.

Imported from March ticket 640. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/640-terminate-trial-pagination-stop-promising-more-results-past-the-end
