---
base: 61815f3a87191d7280c195f5c5f073613f1c4f69
head: 4eedc501f6e08da1034b4c9e7f615bd816f286d3
---
The v0.8.18 review found that invalid article dates can emit backend-looking warnings before the CLI reports a usage error. That makes a simple operator typo look like a network problem and is hard for wrappers to classify.

Imported from March ticket 050. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/050-front-door-article-validation
