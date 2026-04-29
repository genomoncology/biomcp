# Pattern: Drug regulatory and approval evidence

Use this when the question asks for brand-to-generic mapping, approval status, or whether a drug belongs to a target-defined class.

```bash
biomcp get drug "Gocovri" regulatory --region us
biomcp get drug "zanubrutinib" approvals
biomcp get drug "zanubrutinib" targets
biomcp search article --drug "zanubrutinib" -k "second-generation BTK inhibitor" --limit 5
```

Interpretation:
- Let BioMCP normalize brand names first: Gocovri resolves to amantadine, then the U.S. regulatory section answers the FDA-approval question.
- Do not use `biomcp get drug "ADS-5102" all`; current drug cards do not resolve that investigational code, so keep ADS-5102 only as an article or trial-search synonym when needed.
- Split multi-part yes/no questions: approval comes from structured regulatory or approvals sections, while target/class evidence starts with `biomcp get drug "zanubrutinib" targets`.
- Use literature only for context not carried by structured fields, such as first- vs second-generation BTK inhibitor wording.
