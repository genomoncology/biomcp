---
base: d388e242488bc2d8d20d71f857ecde66e6964441
head: eb2b5ad4091ae4e3f3c71afd4aac68a69c6c6602
---
Federated `biomcp search article --source all` averages ~18s (range 13-27s), but a 2026-06-14 deep dive showed nearly all of that residual is one bad source: **litsense2 burns its full 12s per-source cap on every call, sets the federated latency floor, and returns zero results on timeout** (`source_status: degraded, "LitSense2 timed out after 12s"`, observed 100% of calls; standalone it runs ~123s — genuinely broken upstream). The healthy sources (pubtator ~1.7s, europepmc ~1.9s, pubmed ~8.5s) finish concurrently underneath the cap — the fastest federated run was 12.9s, essentially *just* the litsense2 cap.

Imported from March ticket 420. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/420-drop-litsense2-from-source-all-default-set-keep-individually-selectable
