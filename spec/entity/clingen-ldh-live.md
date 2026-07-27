# ClinGen LDH live identity verification

ClinGen Linked Data Hub is an optional identity-verification source for already
retrieved variant articles. This live probe confirms the real medium-to-direct
annotation ladder can contribute an auditable linkage; it does not measure
article discovery or LDH coverage.

## Verify a live LDH article annotation

The request supplies one CAR-applicable CHEK2 identity, so the ladder runs on a
single resolved canonical allele. A successful live response keeps the
provider-labelled linkage and its selector rather than claiming that every
article has LDH coverage.

The linkage below is a specific real one, not an interchangeable example: CAR
resolves `NM_007194.4:c.1100del` to `CA288251`, LDH's literature set for that
allele includes `PMC8710334`, and that article's direct annotation quotes
`rs555607708` once from the article page itself. Most LDH annotations quote a
variant from an article's supplementary spreadsheet instead, which is not an
in-article citation and is deliberately not accepted.

```json file=ldh-chek2.json
[
  {
    "request_id": "chek2-ldh-live",
    "gene": "CHEK2",
    "transcript": "NM_007194.4",
    "coding": "c.1100del"
  }
]
```

```bash
biomcp --json variant articles --input ldh-chek2.json --verify-identity --limit 25 | mustmatch like '"kind": "clingen_ldh_annotation"
"caid": "CA288251"
"gene_id": 11200
"pmcid": "PMC8710334"
"source": "clingen_ldh"
"selector_type": "TextQuoteSelector"
"selector_value": "rs555607708"'
```
