# ClinGen LDH live identity verification

ClinGen Linked Data Hub is an optional identity-verification source for already
retrieved variant articles. This live probe confirms the real medium-to-direct
annotation ladder can contribute an auditable linkage; it does not measure
article discovery or LDH coverage.

## Verify a live LDH article annotation

The request supplies two CAR-applicable ATM identities, so LDH may run only
when their canonical equivalence is confirmed. A successful live response keeps
the provider-labelled linkage and its selector rather than claiming that every
article has LDH coverage.

```json file=ldh-atm.json
[
  {
    "request_id": "atm-ldh-live",
    "gene": "ATM",
    "transcript": "NM_000051.4",
    "coding": "c.1066-6T>G",
    "accession": "NC_000011.10",
    "build": "GRCh38",
    "position": 108248927,
    "ref": "T",
    "alt": "G"
  }
]
```

```bash
biomcp --json variant articles --input ldh-atm.json --verify-identity --limit 10 | mustmatch like '"kind": "clingen_ldh_annotation"
"caid": "CA151456"
"gene_id": 472
"pmcid": "PMC9541484"
"source": "clingen_ldh"
"selector_type":'
```
