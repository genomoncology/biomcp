# Posted Trial Documents and Eligibility Provenance

ClinicalTrials.gov registry eligibility can summarize criteria more briefly than a
posted protocol. BioMCP lists posted document metadata separately, retrieves an
advertised file without conversion, and identifies registry-supplied eligibility
so downstream users can decide when to inspect source documents.

## List posted trial documents

The JSON manifest preserves ClinicalTrials.gov metadata and uses BioMCP commands
as stable retrieval handles. It does not inline or claim to interpret the posted
file.

```bash
set -o pipefail
../../tools/biomcp-ci --json get trial NCT03361748 documents \
  | jq -er 'select((.documents | type) == "array" and (._meta.next_commands | type) == "array") | .nct_id, .source, (.documents[] | select(.filename == "Prot_SAP_000.pdf") | [.type, .label, .date, .upload_date, .size_bytes, .has_protocol, .has_sap, .has_icf, .handle] | @tsv), (._meta.next_commands[] | select(. == "biomcp --json get trial NCT03361748 documents" or . == "biomcp get trial NCT03361748 document Prot_SAP_000.pdf"))' \
  | mustmatch like 'NCT03361748
ClinicalTrials.gov
Prot_SAP	Study Protocol and Statistical Analysis Plan	2019-07-18	2024-12-12T10:49	50	true	true	false	biomcp get trial NCT03361748 document Prot_SAP_000.pdf
biomcp --json get trial NCT03361748 documents
biomcp get trial NCT03361748 document Prot_SAP_000.pdf'
```

## Retrieve one posted trial document

A singular `document` handle returns exactly the advertised bytes. BioMCP does
not parse a PDF or add a text envelope, leaving interpretation to downstream
tools.

```bash
set -o pipefail
../../tools/biomcp-ci get trial NCT03361748 document Prot_SAP_000.pdf | sha256sum | cut -d" " -f1 | mustmatch "e53ed2a9da09c01d1056dd7959cb821be1ff3056d3fad952a63fd956041bfa3e"
```

## Identify registry eligibility provenance

Eligibility JSON distinguishes text copied from the registry from evidence in a
posted document. Document availability is a follow-up signal, not a claim that a
protocol resolves every criterion.

```bash
set -o pipefail
../../tools/biomcp-ci --json get trial NCT03361748 eligibility \
  | jq -er '.eligibility_text, ([.eligibility_provenance.source_kind, .eligibility_provenance.source, .eligibility_provenance.posted_documents_available, .eligibility_provenance.documents_handle] | @tsv)' \
  | mustmatch like 'Inadequate organ function
registry	ClinicalTrials.gov registry	true	biomcp --json get trial NCT03361748 documents'
```

## Follow posted documents from eligibility

The human-readable eligibility card keeps the registry wording and offers a
cautious document follow-up when posted files are available.

```bash
set -o pipefail
../../tools/biomcp-ci get trial NCT03361748 eligibility | mustmatch like 'Inadequate organ function
Posted trial documents
may contain
biomcp --json get trial NCT03361748 documents'
```

## List a trial with no posted documents

A successful manifest can be empty. BioMCP keeps the trial identity and source
while representing the provider's empty document list as an array.

```bash
set -o pipefail
../../tools/biomcp-ci --json get trial NCT41300001 documents \
  | jq -er 'select((.documents | type) == "array" and (.documents | length) == 0) | [.nct_id, .source, (.documents | length)] | @tsv' \
  | mustmatch 'NCT41300001	ClinicalTrials.gov	0'
```

## Identify eligibility when no documents are posted

Registry provenance remains explicit when CTGov has no posted files. The false
availability signal has no document handle and does not erase the criteria.

```bash
set -o pipefail
../../tools/biomcp-ci --json get trial NCT41300001 eligibility \
  | jq -er 'select((.eligibility_provenance | has("documents_handle")) | not) | [.eligibility_text, .eligibility_provenance.source_kind, .eligibility_provenance.source, .eligibility_provenance.posted_documents_available] | @tsv' \
  | mustmatch 'Key inclusion: confirmed SHANK3-related neurodevelopmental disorder.	registry	ClinicalTrials.gov registry	false'
```
