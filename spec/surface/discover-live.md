# Live Discover Checks

These operator-run checks retain discover behaviors that still depend on current upstream responses or credentials. Routine captured OLS4 contracts live in [Discover and Skill](discover.md).

## Trial Suggestions Preserve Resolved Gene Intent

<!-- mustmatch-lint: skip -->

When trial-oriented free text resolves to a gene, `discover` should suggest a
literal biomarker trial search. It must not replace that gene with a curated
disease condition.

```bash run id=discover-gene-trial
../../tools/biomcp-ci discover "SHANK3 clinical trials"
```

```text expect=discover-gene-trial contains
biomcp search trial --biomarker SHANK3 --limit 5
```

```text expect=discover-gene-trial not-contains
Phelan-McDermid
```

## Normalize-to-Codes Playbook Uses Live Discover Code Labels

The normalize-to-codes worked example should teach a real `discover` workflow,
not a copied table of canned codes. The playbook opens the command sequence, and
the live JSON response keeps source-labelled ontology and clinical-code labels
visible for downstream structuring agents when the operator-run verify environment
supplies its configured UMLS credentials.

```bash
../../tools/biomcp-ci skill normalize-to-codes | mustmatch like "biomcp discover
MONDO
SNOMED
ICD-10
RxNorm"
```

```bash
../../tools/biomcp-ci --with-umls-key --json discover "type 2 diabetes mellitus" | mustmatch like '"primary_id": "MONDO:0005148"
"source": "SNOMEDCT"
"source": "ICD10CM"'
```

This credentialed block fails before contacting a provider unless `UMLS_API_KEY` is set. The wrapper preserves only that key for this command and continues stripping every other optional provider credential.
