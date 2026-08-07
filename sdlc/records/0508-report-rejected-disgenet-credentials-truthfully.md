---
base: 497ca27008b366b6c1e5059a8e34b119ce24ab05
head: 2ba4075ecace63892d548012e1b59b5e68a484d8
---
`biomcp --json health --apis-only` can see a configured `DISGENET_API_KEY` and report that DisGeNET rejected it with HTTP 403, but `biomcp --json get gene BRAF disgenet` maps the same response to `api_key_required` and tells the operator to set the already-set variable. This makes a real provider rejection look like local setup omission. Fix the error contract without removing DisGeNET, exposing credentials, or making routine gates depend on a live account.

Imported from March ticket 508. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/508-report-rejected-disgenet-credentials-truthfully
