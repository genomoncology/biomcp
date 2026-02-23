# Pattern: Protein and Pathway Deep Dive

Investigate protein structure, functional domains, interaction partners, and pathway context for a therapeutic target.

## Quick Check

```bash
biomcp get protein P15056 domains
biomcp get pathway R-HSA-5673001
```

## Full Workflow

```bash
# Step 1: Protein profile and function
biomcp get protein P15056

# Step 2: Functional domains
biomcp get protein P15056 domains

# Step 3: Interaction partners
biomcp get protein P15056 interactions

# Step 4: Pathway context
biomcp get gene BRAF pathways
biomcp get pathway R-HSA-5673001

# Step 5: Drugs targeting this pathway
biomcp pathway drugs R-HSA-5673001 --limit 5
```

## Reference

| Source | Provides |
|--------|----------|
| UniProt | Protein sequence, function, domains, interactions |
| PDB/AlphaFold | 3D structure availability |
| Reactome | Pathway membership and events |
| Pathway drugs helper | Approved drugs targeting pathway members |

## Validation Checklist

- [ ] Protein identity and function confirmed
- [ ] Functional domains enumerated
- [ ] Key interaction partners identified
- [ ] Pathway membership established
- [ ] Pathway-targeted drugs listed

## Tips

- Use UniProt accessions (P15056) for proteins; HGNC symbols also work.
- `protein structures` helper lists PDB entries sorted by resolution.
- Combine with variant interpretation: domain-disrupting variants have different significance than surface mutations.
