# BioThings Integration Example Prompts

This guide provides example prompts for AI assistants to effectively use the BioThings suite integration in BioMCP.

## Overview of BioThings Suite

BioMCP integrates with the complete BioThings suite of APIs:

- **MyGene.info** - Gene information and annotations
- **MyDisease.info** - Disease ontology and synonyms
- **MyVariant.info** - Genetic variant annotations (pre-existing integration, enhanced with BioThings client)
- **MyChem.info** - Drug/chemical information and annotations

All four services share common infrastructure through the BioThings client module, providing consistent error handling, rate limiting, and response parsing.

## Gene Information Retrieval

### Basic Gene Lookup

```
"What is the TP53 gene?"
"Tell me about BRAF"
"Get information on the EGFR gene"
"What does the BRCA1 gene do?"
```

**Expected tool usage**: `gene_getter("TP53")` → Returns official name, summary, aliases

### Gene by ID

```
"Look up gene with Entrez ID 7157"
"What is gene 673?"
```

**Expected tool usage**: `gene_getter("7157")` → Returns TP53 information

### Gene Context for Research

```
"I need to understand the KRAS gene before searching for mutations"
"What type of protein does BRAF encode?"
"Give me the official name and aliases for MYC"
```

## Disease Information Retrieval

### Basic Disease Lookup

```
"What is GIST?"
"Tell me about melanoma"
"Define non-small cell lung cancer"
"What is Erdheim-Chester disease?"
```

**Expected tool usage**: `disease_getter("GIST")` → Returns definition, synonyms, ontology IDs

### Disease by Ontology ID

```
"Look up disease MONDO:0018076"
"What is DOID:1909?"
```

**Expected tool usage**: `disease_getter("MONDO:0018076")` → Returns disease information

### Disease Synonyms for Research

```
"What are all the names for gastrointestinal stromal tumor?"
"Find synonyms for NSCLC"
"What other terms are used for melanoma?"
```

## Variant Information Retrieval (MyVariant.info)

MyVariant.info is part of the BioThings suite and provides comprehensive variant annotations. BioMCP has extensive integration with specialized features:

### Basic Variant Lookup

```
"Get information about rs7412"
"What is the BRAF V600E variant?"
"Look up variant chr7:140453136-140453136"
```

**Expected tool usage**: `variant_getter("rs7412")` → Returns variant annotations with external database links

### Variant Search with Filters

```
"Find pathogenic variants in TP53"
"Search for BRCA1 variants with high impact"
"Get all loss-of-function variants in KRAS"
```

**Expected tool usage**: `variant_searcher(gene="TP53", significance="pathogenic")` → Returns filtered variant list

### Variant with Cancer Context

```
"What cancer types have BRAF V600E mutations?"
"Get TCGA data for TP53 R273H"
```

**Expected tool usage**: Variant tools automatically integrate cBioPortal, TCGA, and 1000 Genomes data when available

## Drug Information Retrieval (MyChem.info)

MyChem.info is part of the BioThings suite and provides comprehensive drug/chemical information.

### Basic Drug Lookup

```
"What is imatinib?"
"Tell me about aspirin"
"Get information on pembrolizumab"
"What does metformin do?"
```

**Expected tool usage**: `drug_getter("imatinib")` → Returns drug information with database links

### Drug by ID

```
"Look up DrugBank ID DB00619"
"What is CHEMBL941?"
"Get details for PubChem CID 5291"
```

**Expected tool usage**: `drug_getter("DB00619")` → Returns drug details by identifier

### Drug Properties and Mechanism

```
"What is the mechanism of action of imatinib?"
"Find the chemical formula for aspirin"
"What are the trade names for adalimumab?"
"How does pembrolizumab work?"
```

**Expected tool usage**: `drug_getter("pembrolizumab")` → Returns mechanism, indications, and properties

## Integrated Research Workflows

### Variant Analysis with Gene Context

```
"Analyze the BRAF V600E mutation - first tell me about the gene, then find pathogenic variants"
```

**Expected tool sequence**:

1. `think(thought="Analyzing BRAF V600E mutation", thoughtNumber=1)`
2. `gene_getter("BRAF")` → Gene context
3. `variant_searcher(gene="BRAF", hgvsp="V600E", significance="pathogenic")` → Variant details

### Clinical Trial Search with Disease Expansion

```
"Find clinical trials for GIST patients"
"Search for trials treating gastrointestinal stromal tumors"
```

**Expected tool usage**:

- `trial_searcher(conditions=["GIST"], expand_synonyms=True)`
- Automatically searches for: GIST OR "gastrointestinal stromal tumor" OR "GI stromal tumor"

### Comprehensive Gene-Disease Research

```
"I'm researching EGFR mutations in lung cancer. Start with the gene, then the disease, then find relevant trials"
```

**Expected tool sequence**:

1. `think(thought="Researching EGFR in lung cancer", thoughtNumber=1)`
2. `gene_getter("EGFR")` → Gene information
3. `disease_getter("lung cancer")` → Disease context and synonyms
4. `trial_searcher(conditions=["lung cancer"], interventions=["EGFR inhibitor"])` → Trials with synonym expansion

### Multi-Gene Analysis

```
"Compare TP53, BRAF, and KRAS genes"
"Tell me about the RAS family genes: KRAS, NRAS, HRAS"
```

**Expected tool usage**: Multiple `gene_getter()` calls for each gene

## Advanced Use Cases

### Gene Alias Resolution

```
"What is the official name for the p53 gene?"
"Is TRP53 the same as TP53?"
```

**Expected tool usage**: `gene_getter("p53")` → Will resolve to TP53

### Disease Name Disambiguation

```
"Is GIST the same as gastrointestinal stromal tumor?"
"What's the MONDO ID for melanoma?"
```

**Expected tool usage**: `disease_getter("GIST")` → Shows all synonyms and IDs

### Trial Search Without Synonym Expansion

```
"Find trials specifically mentioning 'GIST' not other names"
```

**Expected tool usage**: `trial_searcher(conditions=["GIST"], expand_synonyms=False)`

### Integrated Literature and Gene Search

```
"Find recent papers about TP53 mutations - first tell me about the gene"
```

**Expected tool sequence**:

1. `gene_getter("TP53")` → Gene context
2. `article_searcher(genes=["TP53"], keywords=["mutation"])` → Literature

### Drug-Target Research

```
"I'm researching imatinib for CML treatment. Get drug info, then find trials"
"What targets does pembrolizumab hit? Then find related articles"
```

**Expected tool sequence**:

1. `think(thought="Researching imatinib for CML", thoughtNumber=1)`
2. `drug_getter("imatinib")` → Drug information and mechanism
3. `trial_searcher(interventions=["imatinib"], conditions=["chronic myeloid leukemia"])`

## Tips for AI Assistants

1. **Always use think() first** for complex biomedical queries
2. **Gene context helps interpretation**: Get gene info before analyzing variants
3. **Disease synonyms improve search**: Use expand_synonyms=True (default) for comprehensive results
4. **Drug mechanisms matter**: Get drug info before searching trials to understand targets
5. **Real-time data**: All BioThings data is fetched live, ensuring current information
6. **Combine tools**: Gene + disease + variant + drug tools work together for comprehensive analysis

## Common Patterns

### Pattern 1: Gene → Variant → Clinical Impact

```
gene_getter("BRAF") →
variant_searcher(gene="BRAF", significance="pathogenic") →
article_searcher(genes=["BRAF"], diseases=["melanoma"])
```

### Pattern 2: Disease → Trials → Locations

```
disease_getter("NSCLC") →
trial_searcher(conditions=["NSCLC"], expand_synonyms=True) →
trial_locations_getter(nct_id="NCT...")
```

### Pattern 3: Multi-Gene Pathway Analysis

```
gene_getter("EGFR") →
gene_getter("KRAS") →
gene_getter("BRAF") →
article_searcher(genes=["EGFR", "KRAS", "BRAF"], keywords=["pathway"])
```

## Unified Search with BioThings Domains

BioMCP's unified search now supports gene, drug, and disease domains alongside articles, trials, and variants:

### Domain-Specific Search

```
"Search for BRAF in the gene domain"
"Find imatinib in drugs"
"Look up melanoma in diseases"
```

**Expected tool usage**:

- `search(domain="gene", keywords=["BRAF"])`
- `search(domain="drug", keywords=["imatinib"])`
- `search(domain="disease", keywords=["melanoma"])`

### Unified Query Language with BioThings

```
"genes.symbol:BRAF AND genes.type:protein-coding"
"drugs.tradename:gleevec"
"diseases.name:melanoma OR diseases.synonym:malignant melanoma"
```

**Expected tool usage**: Query parser automatically routes to appropriate domains

### Cross-Domain Gene Searches

```
"gene:BRAF"  # Searches articles, variants, genes, and trials
"Search everything about TP53"
```

**Expected behavior**:

- Gene queries trigger searches across multiple domains
- Results include gene info, variants, articles, and related trials

### Cross-Domain Disease Searches

```
"disease:melanoma"  # Searches articles, trials, and diseases
"Find all information about NSCLC"
```

**Expected behavior**:

- Disease queries search articles, trials, and disease databases
- Disease synonyms are automatically expanded in trial searches

### Combined Domain Queries

```
"gene:BRAF AND disease:melanoma"
"drugs.indication:leukemia AND trials.phase:3"
"genes.symbol:EGFR AND articles.year:>2023"
```

### Unified Fetch

```
"Fetch BRAF from gene domain"
"Get imatinib details from drugs"
"Retrieve melanoma information from diseases"
```

**Expected tool usage**:

- `fetch(id="BRAF", domain="gene")`
- `fetch(id="imatinib", domain="drug")`
- `fetch(id="melanoma", domain="disease")`

## Error Handling

If a gene/disease is not found:

- Check for typos or alternative names
- Try searching with partial names
- Use official symbols for genes (e.g., "TP53" not "p53 gene")
- For diseases, try both common and medical names
