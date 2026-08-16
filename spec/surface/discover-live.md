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
set -o pipefail
../../tools/biomcp-ci --with-umls-key --json discover "Diabetes Mellitus, Non-Insulin-Dependent" --full |
  env -i PATH="$PATH" python3 /dev/fd/3 3<<'PY'
import json
import os
import sys


def fail(message):
    raise SystemExit(message)


provider_credentials = {
    "UMLS_API_KEY",
    "NCBI_API_KEY",
    "S2_API_KEY",
    "OPENFDA_API_KEY",
    "NCI_API_KEY",
    "ONCOKB_TOKEN",
    "DISGENET_API_KEY",
    "ALPHAGENOME_API_KEY",
}
leaked_credentials = provider_credentials.intersection(os.environ)
if leaked_credentials:
    fail(f"provider credentials leaked to parser: {sorted(leaked_credentials)}")

document = json.load(sys.stdin)
if not isinstance(document, dict):
    fail("discover response must be a JSON object")
concepts = document.get("concepts")
if not isinstance(concepts, list) or not all(isinstance(item, dict) for item in concepts):
    fail("discover concepts must be an array of objects")


def exact_concept(primary_id):
    matches = [item for item in concepts if item.get("primary_id") == primary_id]
    if len(matches) != 1:
        fail(f"expected exactly one {primary_id} concept, found {len(matches)}")
    return matches[0]


exact_concept("MONDO:0005148")
umls = exact_concept("UMLS:C0011860")

sources = umls.get("sources")
if not isinstance(sources, list) or not sources:
    fail("UMLS:C0011860 sources must be a non-empty array")
source_names = []
for source in sources:
    if not isinstance(source, dict) or not isinstance(source.get("source"), str):
        fail("UMLS:C0011860 has a malformed source")
    source_names.append(source["source"].strip().upper())
if "UMLS" not in source_names:
    fail("UMLS:C0011860 is missing its UMLS source")

xrefs = umls.get("xrefs")
if not isinstance(xrefs, dict) or not isinstance(xrefs.get("values"), list):
    fail("UMLS:C0011860 xrefs must contain a values array")
xref_sources = []
for xref in xrefs["values"]:
    if not isinstance(xref, dict) or not isinstance(xref.get("source"), str):
        fail("UMLS:C0011860 has a malformed xref source")
    xref_sources.append(xref["source"].strip().upper())
if not any(source.startswith("SNOMEDCT") for source in xref_sources):
    fail("UMLS:C0011860 is missing a SNOMEDCT-family xref")
if not any(source.startswith("ICD10") for source in xref_sources):
    fail("UMLS:C0011860 is missing an ICD10-family xref")

print("live UMLS discovery identities and code families verified")
PY
```

This credentialed block fails before contacting a provider unless `UMLS_API_KEY` is set. The wrapper preserves only that key for this command and continues stripping every other optional provider credential.
