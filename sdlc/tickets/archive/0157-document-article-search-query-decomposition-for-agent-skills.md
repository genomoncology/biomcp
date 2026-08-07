---
flow: build
priority: 3
---
# Document article search query decomposition for agent skills

BioMCP's article search quality depends on how the agent decomposes a question into CLI parameters. Research 011 showed a 7x recall improvement when using structured entity fields (`--gene`, `--disease`, `--drug`) plus cleaned keywords versus passing the raw NL question as `--keyword`. But the agent guidance for how to formulate these queries doesn't exist in the CLI help, docs, or reference material that agents consume.

Completed under March on 2026-04-09, as March ticket 157. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/157-document-article-search-query-decomposition-for-agent-skills
