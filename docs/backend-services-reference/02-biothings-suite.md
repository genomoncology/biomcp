# BioThings Suite API Reference

The BioThings Suite provides unified access to biomedical annotations across genes, variants, diseases, and drugs through a consistent API interface.

## Usage Examples

For practical examples using the BioThings APIs, see:

- [How to Find Trials with NCI and BioThings](../how-to-guides/02-find-trials-with-nci-and-biothings.md#biothings-integration-for-enhanced-search)
- [Get Comprehensive Variant Annotations](../how-to-guides/03-get-comprehensive-variant-annotations.md#integration-with-other-biomcp-tools)

## Overview

BioMCP integrates with four BioThings APIs:

- **MyGene.info**: Gene annotations and functional information
- **MyVariant.info**: Genetic variant annotations and clinical significance
- **MyDisease.info**: Disease ontology and terminology mappings
- **MyChem.info**: Drug/chemical properties and mechanisms

All APIs share:

- RESTful JSON interface
- No authentication required
- Elasticsearch-based queries
- Comprehensive data aggregation

## MyGene.info

### Base URL

`https://mygene.info/v1/`

### Key Endpoints

#### Gene Query

```
GET /query?q={query}
```

**Parameters:**

- `q`: Query string (gene symbol, name, or ID)
- `fields`: Specific fields to return
- `species`: Limit to species (default: human, mouse, rat)
- `size`: Number of results (default: 10)

**Example:**

```bash
curl "https://mygene.info/v1/query?q=BRAF&fields=symbol,name,summary,type_of_gene"
```

#### Gene Annotation

```
GET /gene/{geneid}
```

**Gene ID formats:**

- Entrez Gene ID: `673`
- Ensembl ID: `ENSG00000157764`
- Gene Symbol: `BRAF`

**Example:**

```bash
curl "https://mygene.info/v1/gene/673?fields=symbol,name,summary,genomic_pos,pathway,go"
```

### Important Fields

| Field         | Description            | Example                                 |
| ------------- | ---------------------- | --------------------------------------- |
| `symbol`      | Official gene symbol   | "BRAF"                                  |
| `name`        | Full gene name         | "B-Raf proto-oncogene"                  |
| `entrezgene`  | NCBI Entrez ID         | 673                                     |
| `summary`     | Functional description | "This gene encodes..."                  |
| `genomic_pos` | Chromosomal location   | {"chr": "7", "start": 140433812}        |
| `pathway`     | Pathway memberships    | {"kegg": [...], "reactome": [...]}      |
| `go`          | Gene Ontology terms    | {"BP": [...], "MF": [...], "CC": [...]} |

## MyVariant.info

### Base URL

`https://myvariant.info/v1/`

### Key Endpoints

#### Variant Query

```
GET /query?q={query}
```

**Query syntax:**

- Gene + variant: `dbnsfp.genename:BRAF AND dbnsfp.hgvsp:p.V600E`
- rsID: `dbsnp.rsid:rs121913529`
- Genomic: `_id:chr7:g.140453136A>T`

**Example:**

```bash
curl "https://myvariant.info/v1/query?q=dbnsfp.genename:TP53&fields=_id,clinvar,gnomad_exome"
```

#### Variant Annotation

```
GET /variant/{variant_id}
```

**ID formats:**

- HGVS genomic: `chr7:g.140453136A>T`
- dbSNP: `rs121913529`

### Important Fields

| Field          | Description            | Example                                 |
| -------------- | ---------------------- | --------------------------------------- |
| `clinvar`      | Clinical significance  | {"clinical_significance": "Pathogenic"} |
| `dbsnp`        | dbSNP annotations      | {"rsid": "rs121913529"}                 |
| `cadd`         | CADD scores            | {"phred": 35}                           |
| `gnomad_exome` | Population frequency   | {"af": {"af": 0.00001}}                 |
| `dbnsfp`       | Functional predictions | {"polyphen2": "probably_damaging"}      |

### Query Filters

```python
# Clinical significance
q = "clinvar.clinical_significance:pathogenic"

# Frequency filters
q = "gnomad_exome.af.af:<0.01"  # Rare variants

# Gene-specific
q = "dbnsfp.genename:BRCA1 AND cadd.phred:>20"
```

## MyDisease.info

### Base URL

`https://mydisease.info/v1/`

### Key Endpoints

#### Disease Query

```
GET /query?q={query}
```

**Example:**

```bash
curl "https://mydisease.info/v1/query?q=melanoma&fields=mondo,disease_ontology,synonyms"
```

#### Disease Annotation

```
GET /disease/{disease_id}
```

**ID formats:**

- MONDO: `MONDO:0007254`
- DOID: `DOID:1909`
- OMIM: `OMIM:155600`

### Important Fields

| Field              | Description       | Example                                      |
| ------------------ | ----------------- | -------------------------------------------- |
| `mondo`            | MONDO ontology    | {"id": "MONDO:0007254", "label": "melanoma"} |
| `disease_ontology` | Disease Ontology  | {"id": "DOID:1909"}                          |
| `synonyms`         | Alternative names | ["malignant melanoma", "MM"]                 |
| `xrefs`            | Cross-references  | {"omim": ["155600"], "mesh": ["D008545"]}    |
| `phenotypes`       | HPO terms         | [{"hpo_id": "HP:0002861"}]                   |

## MyChem.info

### Base URL

`https://mychem.info/v1/`

### Key Endpoints

#### Drug Query

```
GET /query?q={query}
```

**Example:**

```bash
curl "https://mychem.info/v1/query?q=imatinib&fields=drugbank,chembl,chebi"
```

#### Drug Annotation

```
GET /drug/{drug_id}
```

**ID formats:**

- DrugBank: `DB00619`
- ChEMBL: `CHEMBL941`
- Name: `imatinib`

### Important Fields

| Field          | Description    | Example                                      |
| -------------- | -------------- | -------------------------------------------- |
| `drugbank`     | DrugBank data  | {"id": "DB00619", "name": "Imatinib"}        |
| `chembl`       | ChEMBL data    | {"molecule_chembl_id": "CHEMBL941"}          |
| `chebi`        | ChEBI ontology | {"id": "CHEBI:45783"}                        |
| `drugcentral`  | Indications    | {"indications": [...]}                       |
| `pharmacology` | Mechanism      | {"mechanism_of_action": "BCR-ABL inhibitor"} |

## Common Query Patterns

### 1. Gene to Variant Pipeline

```python
# Step 1: Get gene info
gene_response = requests.get(
    "https://mygene.info/v1/gene/BRAF",
    params={"fields": "symbol,genomic_pos"}
)

# Step 2: Find variants in gene
variant_response = requests.get(
    "https://myvariant.info/v1/query",
    params={
        "q": "dbnsfp.genename:BRAF",
        "fields": "clinvar.clinical_significance,gnomad_exome.af",
        "size": 100
    }
)
```

### 2. Disease Synonym Expansion

```python
# Get all synonyms for a disease
disease_response = requests.get(
    "https://mydisease.info/v1/query",
    params={
        "q": "melanoma",
        "fields": "mondo,synonyms,xrefs"
    }
)

# Extract all names
all_names = ["melanoma"]
for hit in disease_response.json()["hits"]:
    if "synonyms" in hit:
        all_names.extend(hit["synonyms"])
```

### 3. Drug Target Lookup

```python
# Find drugs targeting a gene
drug_response = requests.get(
    "https://mychem.info/v1/query",
    params={
        "q": "drugcentral.targets.gene_symbol:BRAF",
        "fields": "drugbank.name,chembl.pref_name",
        "size": 50
    }
)
```

## Rate Limits and Best Practices

### Rate Limits

- **Default**: 1,000 requests/hour per IP
- **Batch queries**: Up to 1,000 IDs per request
- **No authentication**: Public access

### Best Practices

#### 1. Use Field Filtering

```python
# Good - only request needed fields
params = {"fields": "symbol,name,summary"}

# Bad - returns all fields
params = {}
```

#### 2. Batch Requests

```python
# Good - single request for multiple genes
response = requests.post(
    "https://mygene.info/v1/gene",
    json={"ids": ["BRAF", "KRAS", "EGFR"]}
)

# Bad - multiple individual requests
for gene in ["BRAF", "KRAS", "EGFR"]:
    requests.get(f"https://mygene.info/v1/gene/{gene}")
```

#### 3. Handle Missing Data

```python
# Check for field existence
if "clinvar" in variant and "clinical_significance" in variant["clinvar"]:
    significance = variant["clinvar"]["clinical_significance"]
else:
    significance = "Not available"
```

## Error Handling

### Common Errors

#### 404 Not Found

```json
{
  "success": false,
  "error": "ID not found"
}
```

#### 400 Bad Request

```json
{
  "success": false,
  "error": "Invalid query syntax"
}
```

#### 429 Rate Limited

```json
{
  "success": false,
  "error": "Rate limit exceeded"
}
```

### Error Handling Code

```python
def query_biothings(api_url, query_params):
    try:
        response = requests.get(api_url, params=query_params)
        response.raise_for_status()
        return response.json()
    except requests.exceptions.HTTPError as e:
        if e.response.status_code == 404:
            return {"error": "Not found", "query": query_params}
        elif e.response.status_code == 429:
            # Implement exponential backoff
            time.sleep(60)
            return query_biothings(api_url, query_params)
        else:
            raise
```

## Data Sources

Each BioThings API aggregates data from multiple sources:

### MyGene.info Sources

- NCBI Entrez Gene
- Ensembl
- UniProt
- KEGG, Reactome, WikiPathways
- Gene Ontology

### MyVariant.info Sources

- dbSNP
- ClinVar
- gnomAD
- CADD
- PolyPhen-2, SIFT
- COSMIC

### MyDisease.info Sources

- MONDO
- Disease Ontology
- OMIM
- MeSH
- HPO

### MyChem.info Sources

- DrugBank
- ChEMBL
- ChEBI
- PubChem
- DrugCentral

## Advanced Features

### Full-Text Search

```python
# Search across all fields
params = {
    "q": "lung cancer EGFR",  # Searches all text fields
    "fields": "symbol,name,summary"
}
```

### Faceted Search

```python
# Get aggregations
params = {
    "q": "clinvar.clinical_significance:pathogenic",
    "facets": "dbnsfp.genename",
    "size": 0  # Only return facets
}
```

### Scrolling Large Results

```python
# For results > 10,000
params = {
    "q": "dbnsfp.genename:TP53",
    "fetch_all": True,
    "fields": "_id"
}
```

## Integration Tips

### 1. Caching Strategy

- Cache gene/drug/disease lookups (stable)
- Don't cache variant queries (frequently updated)
- Use ETags for conditional requests

### 2. Parallel Requests

```python
import asyncio
import aiohttp

async def fetch_all(session, urls):
    tasks = []
    for url in urls:
        tasks.append(session.get(url))
    return await asyncio.gather(*tasks)
```

### 3. Data Normalization

```python
def normalize_gene_symbol(symbol):
    # Query MyGene to get official symbol
    response = requests.get(
        f"https://mygene.info/v1/query?q={symbol}"
    )
    if response.json()["hits"]:
        return response.json()["hits"][0]["symbol"]
    return symbol
```
