# BioMCP Source Contract Probes

This note captures BioMCP source-facing API contract probes for the current
operator path. Each source includes one happy query, one edge/empty query, and
one invalid query.

## UniProt

- Happy
```bash
curl -sS -L "https://rest.uniprot.org/uniprotkb/P15056.json"
```
- Edge (valid accession pattern, expected 404)
```bash
curl -sS -L "https://rest.uniprot.org/uniprotkb/P99999.json"
```
- Invalid
```bash
curl -sS -L "https://rest.uniprot.org/uniprotkb/P15056.not-real"
```

## QuickGO

- Happy
```bash
curl -sS -L "https://www.ebi.ac.uk/QuickGO/services/annotation/search?geneProductId=P15056&limit=5"
```
- Edge
```bash
curl -sS -L "https://www.ebi.ac.uk/QuickGO/services/annotation/search?geneProductId=P99999&limit=5"
```
- Invalid
```bash
curl -sS -L "https://www.ebi.ac.uk/QuickGO/services/annotation/search?geneProductId=P15056&limit=-1"
```

## STRING

- Happy
```bash
curl -sS -L "https://string-db.org/api/json/network?identifiers=BRAF&species=9606&limit=5"
```
- Edge
```bash
curl -sS -L "https://string-db.org/api/json/network?identifiers=NO_SUCH_GENE_091&species=9606&limit=5"
```
- Invalid
```bash
curl -sS -L "https://string-db.org/api/json/not-a-real-endpoint?identifiers=BRAF&species=9606"
```

## gnomAD GraphQL

- Happy
```bash
curl -sS -L -X POST "https://gnomad.broadinstitute.org/api" \
  -H "content-type: application/json" \
  -d '{"query":"query { __typename }"}'
```
- Edge
```bash
curl -sS -L -X POST "https://gnomad.broadinstitute.org/api" \
  -H "content-type: application/json" \
  -d '{"query":"query { variant(variantId: \"7-1-A-T\", dataset: gnomad_r4) { variantId } }"}'
```
- Invalid
```bash
curl -sS -L -X POST "https://gnomad.broadinstitute.org/api" \
  -H "content-type: application/json" \
  -d '{"query":"query { variant(variantId: \"bad\") {"}'
```

## ChEMBL

- Happy
```bash
curl -sS -L "https://www.ebi.ac.uk/chembl/api/data/molecule/CHEMBL25.json"
```
- Edge
```bash
curl -sS -L "https://www.ebi.ac.uk/chembl/api/data/molecule/search.json?q=NOT_A_REAL_DRUG_091"
```
- Invalid
```bash
curl -sS -L "https://www.ebi.ac.uk/chembl/api/data/not-a-real-resource.json"
```

## OpenTargets GraphQL

- Happy
```bash
curl -sS -L -X POST "https://api.platform.opentargets.org/api/v4/graphql" \
  -H "content-type: application/json" \
  -d '{"query":"query { drug(chemblId: \"CHEMBL25\") { id name } }"}'
```
- Edge
```bash
curl -sS -L -X POST "https://api.platform.opentargets.org/api/v4/graphql" \
  -H "content-type: application/json" \
  -d '{"query":"query { drug(chemblId: \"CHEMBL_DOES_NOT_EXIST\") { id name } }"}'
```
- Invalid (wrong argument key)
```bash
curl -sS -L -X POST "https://api.platform.opentargets.org/api/v4/graphql" \
  -H "content-type: application/json" \
  -d '{"query":"query { drug(efoId: \"EFO_0000311\") { id name } }"}'
```

## Reactome

- Happy
```bash
curl -sS -L "https://reactome.org/ContentService/search/query?query=MAPK&species=Homo%20sapiens&pageSize=1"
```
- Edge
```bash
curl -sS -L "https://reactome.org/ContentService/search/query?query=NO_SUCH_PATHWAY_091&species=Homo%20sapiens&pageSize=1"
```
- Invalid
```bash
curl -sS -L "https://reactome.org/ContentService/data/query/NOT_A_REAL_STABLE_ID"
```

## KEGG

- Happy search
```bash
curl -sS -L "https://rest.kegg.jp/find/pathway/MAPK"
```
- Happy get
```bash
curl -sS -L "https://rest.kegg.jp/get/hsa04010"
```
- No-hit
```bash
curl -sS -L "https://rest.kegg.jp/find/pathway/NO_SUCH_PATHWAY_091"
```

## g:Profiler

- Happy
```bash
curl -sS -L -X POST "https://biit.cs.ut.ee/gprofiler/api/gost/profile/" \
  -H "content-type: application/json" \
  -d '{"organism":"hsapiens","query":["BRAF","KRAS"]}'
```
- Edge
```bash
curl -sS -L -X POST "https://biit.cs.ut.ee/gprofiler/api/gost/profile/" \
  -H "content-type: application/json" \
  -d '{"organism":"hsapiens","query":["NO_SUCH_GENE_091"]}'
```
- Invalid
```bash
curl -sS -L -X POST "https://biit.cs.ut.ee/gprofiler/api/gost/profile/" \
  -H "content-type: application/json" \
  -d '{"query":"not-an-array"}'
```

## HPA

- Happy
```bash
curl -sS -L "https://www.proteinatlas.org/ENSG00000157764.xml"
```
- Missing gene
```bash
curl -sS -L "https://www.proteinatlas.org/ENSG00000999999.xml"
```

## InterPro

- Happy
```bash
curl -sS -L "https://www.ebi.ac.uk/interpro/api/entry/interpro/protein/uniprot/P15056/?page_size=5"
```
- Edge
```bash
curl -sS -L "https://www.ebi.ac.uk/interpro/api/entry/interpro/protein/uniprot/P99999/?page_size=5"
```
- Invalid
```bash
curl -sS -L "https://www.ebi.ac.uk/interpro/api/entry/interpro/protein/uniprot/P15056/not-a-real-resource/"
```

## ComplexPortal

- Happy
```bash
curl -sS -L "https://www.ebi.ac.uk/intact/complex-ws/search/P15056?number=25&filters=species_f:(%22Homo%20sapiens%22)"
```
- No-match
```bash
curl -sS -L "https://www.ebi.ac.uk/intact/complex-ws/search/NO_SUCH_PROTEIN_091?number=25&filters=species_f:(%22Homo%20sapiens%22)"
```

## Semantic Scholar (optional, requires `S2_API_KEY`)

Keep this probe bounded because the approved key tier is 1 request / second.
Skip it cleanly when `S2_API_KEY` is absent.

- Happy detail
```bash
curl -sS -L "https://api.semanticscholar.org/graph/v1/paper/PMID:22663011?fields=paperId,title" \
  -H "x-api-key: $S2_API_KEY"
```
- Happy citation graph
```bash
curl -sS -L "https://api.semanticscholar.org/graph/v1/paper/PMID:22663011/citations?fields=contexts,intents,isInfluential&limit=1" \
  -H "x-api-key: $S2_API_KEY"
```

## ClinicalTrials.gov (existing variant-support source)

- Happy
```bash
curl -sS -L "https://clinicaltrials.gov/api/v2/studies?query.term=BRAF%20V600E&pageSize=1"
```
- Edge
```bash
curl -sS -L "https://clinicaltrials.gov/api/v2/studies?query.term=NO_SUCH_TERM_091&pageSize=1"
```
- Invalid
```bash
curl -sS -L "https://clinicaltrials.gov/api/v2/studies?query.term=melanoma&pageSize=bad"
```

## cBioPortal (existing variant-support source)

- Happy
```bash
curl -sS -L "https://www.cbioportal.org/api/studies?projection=SUMMARY&pageSize=1"
```
- Edge
```bash
curl -sS -L "https://www.cbioportal.org/api/molecular-profiles?studyId=NO_SUCH_STUDY"
```
- Invalid
```bash
curl -sS -L "https://www.cbioportal.org/api/not-a-real-resource"
```

## OncoKB (existing variant-support source)

- Happy (demo, no token)
```bash
curl -sS -L "https://demo.oncokb.org/api/v1/annotate/mutations/byProteinChange?hugoSymbol=BRAF&alteration=V600E"
```
- Edge (known no-hit demo response)
```bash
curl -sS -L "https://demo.oncokb.org/api/v1/annotate/mutations/byProteinChange?hugoSymbol=EGFR&alteration=L858R"
```
- Additional test gene
```bash
curl -sS -L "https://demo.oncokb.org/api/v1/annotate/mutations/byProteinChange?hugoSymbol=TP53&alteration=R175H"
```

- Production probe (requires `ONCOKB_TOKEN`)
```bash
curl -sS -L "https://www.oncokb.org/api/v1/annotate/mutations/byProteinChange?hugoSymbol=BRAF&alteration=V600E" \
  -H "Authorization: Bearer $ONCOKB_TOKEN"
```
