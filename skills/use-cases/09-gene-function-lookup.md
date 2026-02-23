# Pattern: Gene Function Lookup

Resolve gene identity and summarize function, disease relevance, and therapeutic context.

> Inspired by GeneGPT-style gene-centric tasks.

## Quick Check

```bash
biomcp get gene BRAF
biomcp get variant rs113488022 clinvar
```

## Full Workflow

```bash
# Step 1: Resolve symbol and aliases
biomcp get gene BRAF

# Step 2: Protein context
biomcp get gene BRAF protein

# Step 2.5: Protein structure and domains
biomcp get protein P15056 domains

# Step 3: Map representative rsID/variant context
biomcp get variant rs113488022 clinvar

# Step 4: Add pathway and disease relevance
biomcp get gene BRAF pathways
biomcp get gene BRAF diseases

# Step 5: Pull treatment-linked evidence
biomcp search article -g BRAF -d melanoma --drug dabrafenib --limit 5
```

## Reference

| Source | Provides |
|--------|----------|
| Gene entity | Aliases, identifiers, location, summary |
| Variant ClinVar | Clinical relevance for representative variant |
| Pathway/disease sections | Functional and disease context |
| UniProt | Protein structure and domains |
| Article search | Treatment-linked evidence trail |

## Validation Checklist

A complete run should produce:

- [ ] Canonical gene identity and aliases
- [ ] Location/identifier context for the gene
- [ ] Variant mapping linked to the gene
- [ ] Pathway and disease context
- [ ] Therapy-linked evidence references

## Tips

- Start with gene symbol; use aliases only when symbol is uncertain.
- Add section-specific requests (`pathways`, `diseases`) to reduce output noise.
- Keep one representative variant to anchor clinical interpretation.
