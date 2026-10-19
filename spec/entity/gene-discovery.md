# Gene Discovery and Identity

Gene search anchors a session in a stable entity before readers open optional sections.

## Symbol-Based Search

Symbol search should still surface the canonical BRAF row in a human-scannable
table before the user pivots into deeper sections.

```bash
../../tools/biomcp-ci search gene BRAF --limit 3 | mustmatch like '# Genes: BRAF
B-Raf proto-oncogene'
```

## Identity Card

The default card should keep the persistent identifier and the progressive
disclosure hints that let readers deepen into the right follow-up section.

```bash
../../tools/biomcp-ci get gene BRAF | mustmatch like 'Entrez ID: 673
biomcp get gene BRAF pathways
biomcp get gene BRAF diagnostics'
```
