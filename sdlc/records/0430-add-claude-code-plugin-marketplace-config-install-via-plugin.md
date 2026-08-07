---
base: 3c4da3098f59a77b7ca835529afca0f72e800642
head: 16888d7fd14d8c025106eee7ef0349a1b4bb77f7
---
A contributor (renato-umeton) prototyped this on a fork but can't open a PR. This ticket lands a correct, schema-current version in-repo. Note: BioMCP **already** has a "skill" system (`biomcp skill install`, the `skills/` directory) — that is unrelated to this. The actual gap is the Claude Code **plugin marketplace** packaging so the `/plugin` flow works.

Imported from March ticket 430. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/430-add-claude-code-plugin-marketplace-config-install-via-plugin
