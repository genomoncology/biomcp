# Pattern: Disease locus and chromosome mapping

Use this when the question asks which chromosomes, loci, deletions, duplications, or trisomies are linked to a disease.

```bash
biomcp search article -k "Arnold Chiari syndrome chromosome" --type review --limit 10
biomcp batch article 39309470,17103432,12210325 --mode compact
biomcp search article -k "\"Arnold Chiari\" deletion duplication trisomy chromosome" --limit 10
biomcp batch article 12522795,15742475,29410707 --mode compact
```

Interpretation:
- Start broad with review literature when the question is about locus-level history.
- Batch several candidate papers before deciding which chromosome claims are supported.
- Treat case reports and cytogenetic findings as evidence to verify, not as a complete map.
- Separate chromosome or locus findings from named-gene causality unless the paper states both.
