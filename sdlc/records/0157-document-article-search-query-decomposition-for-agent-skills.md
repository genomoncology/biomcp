---
base: 58520458068595f29d0770351e6172e75d51d002
head: 401f41460603b3f7cc689d1f8d45aef4e114ea2a
---
BioMCP's article search quality depends on how the agent decomposes a question into CLI parameters. Research 011 showed a 7x recall improvement when using structured entity fields (`--gene`, `--disease`, `--drug`) plus cleaned keywords versus passing the raw NL question as `--keyword`. But the agent guidance for how to formulate these queries doesn't exist in the CLI help, docs, or reference material that agents consume.

Imported from March ticket 157. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/157-document-article-search-query-decomposition-for-agent-skills
