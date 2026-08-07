---
base: 7ae10dc50b8d330486c2c20fc77b58a1dec0620a
head: 0719255b30f7932915e16c7f426d641263344478
---
BioMCP needs a local knowledge-base feature that turns biomedical content into durable, searchable Markdown files. Obsidian is the target front-end — its vaults are ordinary folders, its CLI supports search/create/read, and its frontmatter is agent-friendly. Before building the full feature, we need hands-on validation of the key technical unknowns: Obsidian CLI integration, Rust PDF-to-Markdown quality, JATS XML conversion, and HTML extraction.

Imported from March ticket 244. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/244-spike-obsidian-vault-knowledge-base-with-pdf-to-markdown
