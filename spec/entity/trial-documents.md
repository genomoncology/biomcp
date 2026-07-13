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
../../tools/biomcp-ci --json get trial NCT03361748 documents \
  | jq -r '.nct_id, .source, (.documents[]? | select(.filename == "Prot_SAP_000.pdf") | [.type, .label, .date, .upload_date, .size_bytes, .has_protocol, .has_sap, .has_icf, .handle] | @tsv), (._meta.next_commands[]? | select(. == "biomcp get trial NCT03361748 document Prot_SAP_000.pdf"))' \
  | mustmatch like 'NCT03361748
ClinicalTrials.gov
Prot_SAP	Study Protocol and Statistical Analysis Plan	2019-07-18	2024-12-12T10:49	50	true	true	false	biomcp get trial NCT03361748 document Prot_SAP_000.pdf
biomcp get trial NCT03361748 document Prot_SAP_000.pdf'
```

## Retrieve one posted trial document

A singular `document` handle returns exactly the advertised bytes. BioMCP does
not parse a PDF or add a text envelope, leaving interpretation to downstream
tools.

```bash
../../tools/biomcp-ci get trial NCT03361748 document Prot_SAP_000.pdf | sha256sum | cut -d" " -f1 | mustmatch "e53ed2a9da09c01d1056dd7959cb821be1ff3056d3fad952a63fd956041bfa3e"
```

## Identify registry eligibility provenance

Eligibility JSON distinguishes text copied from the registry from evidence in a
posted document. Document availability is a follow-up signal, not a claim that a
protocol resolves every criterion.

```bash
../../tools/biomcp-ci --json get trial NCT03361748 eligibility \
  | jq -r '.eligibility_text, ([.eligibility_provenance.source_kind, .eligibility_provenance.source, .eligibility_provenance.posted_documents_available, .eligibility_provenance.documents_handle] | @tsv)' \
  | mustmatch like 'Inadequate organ function
registry	ClinicalTrials.gov registry	true	biomcp --json get trial NCT03361748 documents'
```

## Follow posted documents from eligibility

The human-readable eligibility card keeps the registry wording and offers a
cautious document follow-up when posted files are available.

```bash
../../tools/biomcp-ci get trial NCT03361748 eligibility | mustmatch like 'Inadequate organ function
Posted trial documents are available and may contain additional eligibility detail: `biomcp --json get trial NCT03361748 documents`'
```
