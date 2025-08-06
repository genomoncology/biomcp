# How to Find Trials with NCI and BioThings

This guide demonstrates how to search for clinical trials using BioMCP's dual data sources and automatic disease synonym expansion.

## Overview

BioMCP provides access to clinical trials through:

- **ClinicalTrials.gov**: Default source with comprehensive U.S. and international trials ([API Reference](../backend-services-reference/04-clinicaltrials-gov.md))
- **NCI CTS API**: Advanced cancer trial search with biomarker filtering (requires API key) ([API Reference](../backend-services-reference/05-nci-cts-api.md))
- **BioThings Integration**: Automatic disease synonym expansion for better coverage ([BioThings Reference](../backend-services-reference/02-biothings-suite.md))

## Basic Trial Search

### Simple Disease Search

Find trials for a specific condition:

```bash
# CLI
biomcp trial search --condition melanoma --status RECRUITING

# Python
trials = await client.trials.search(
    conditions=["melanoma"],
    recruiting_status="RECRUITING"
)

# MCP Tool
trial_searcher(
    conditions=["melanoma"],
    recruiting_status="OPEN"
)
```

### Search by Intervention

Find trials testing specific drugs:

```bash
# CLI
biomcp trial search --intervention pembrolizumab --phase PHASE3

# Python
trials = await client.trials.search(
    interventions=["pembrolizumab"],
    phase="PHASE3"
)
```

## Location-Based Search

### Finding Nearby Trials

**Important**: Location searches require latitude and longitude coordinates.

```python
# Find trials near Cleveland, Ohio
trials = await trial_searcher(
    conditions=["lung cancer"],
    lat=41.4993,
    long=-81.6944,
    distance=50  # 50 miles radius
)

# Find trials near Boston
trials = await trial_searcher(
    conditions=["breast cancer"],
    lat=42.3601,
    long=-71.0589,
    distance=25
)
```

### Getting Coordinates

For common locations:

- Cleveland: lat=41.4993, long=-81.6944
- Boston: lat=42.3601, long=-71.0589
- New York: lat=40.7128, long=-74.0060
- Los Angeles: lat=34.0522, long=-118.2437
- Houston: lat=29.7604, long=-95.3698

## Advanced Filtering

### Multiple Criteria

Combine multiple filters for precise results:

```python
# Complex search example
trials = await trial_searcher(
    conditions=["non-small cell lung cancer", "NSCLC"],
    interventions=["pembrolizumab", "immunotherapy"],
    phase="PHASE3",
    recruiting_status="OPEN",
    age_group="ADULT",
    study_type="INTERVENTIONAL",
    funder_type="INDUSTRY"
)
```

### Date-Based Filtering

Find recently started trials:

```bash
# CLI - Trials started in 2024
biomcp trial search \
  --condition cancer \
  --start-date 2024-01-01 \
  --status RECRUITING
```

## Using NCI API Advanced Features

### Setup NCI API Key

Get your key from [api.cancer.gov](https://api.cancer.gov). For detailed setup instructions, see [Authentication and API Keys](../getting-started/03-authentication-and-api-keys.md#nci-clinical-trials-api):

```bash
export NCI_API_KEY="your-key-here"
```

### Biomarker-Based Search

Find trials for specific mutations:

```python
# Search using NCI source
trials = await search(
    domain="trial",
    source="nci",
    conditions=["melanoma"],
    required_mutations=["BRAF V600E"],
    allow_brain_mets=True,
    api_key="your-key"
)
```

### NCI-Specific Parameters

```python
# Advanced NCI search
trials = await trial_searcher(
    source="nci",
    conditions=["lung cancer"],
    required_mutations=["EGFR L858R", "EGFR exon 19 deletion"],
    prior_therapy_required=False,
    allow_brain_mets=True,
    allow_prior_immunotherapy=False,
    api_key="your-key"
)
```

## BioThings Integration for Enhanced Search

For technical details on the BioThings APIs, see:

- [BioThings Suite Reference](../backend-services-reference/02-biothings-suite.md)

### Automatic Disease Synonym Expansion

BioMCP automatically expands disease terms using MyDisease.info:

```python
# Searching for "GIST" automatically includes:
# - "gastrointestinal stromal tumor"
# - "gastrointestinal stromal tumour"
# - "GI stromal tumor"
trials = await trial_searcher(conditions=["GIST"])
```

### Manual Disease Lookup

Get all synonyms for a disease:

```python
# Get disease information
disease_info = await disease_getter("melanoma")

# Extract synonyms
synonyms = disease_info.synonyms
# Returns: ["malignant melanoma", "melanoma, malignant", ...]

# Use in trial search
trials = await trial_searcher(conditions=synonyms)
```

## Practical Workflows

### Workflow 1: Patient-Centric Trial Search

Find trials for a specific patient profile:

```python
async def find_trials_for_patient(
    disease: str,
    mutations: list[str],
    location: tuple[float, float],
    prior_treatments: list[str]
):
    # Step 1: Think about the search
    await think(
        thought=f"Searching trials for {disease} with {mutations}",
        thoughtNumber=1
    )

    # Step 2: Get disease synonyms
    disease_info = await disease_getter(disease)
    all_conditions = [disease] + disease_info.synonyms

    # Step 3: Search both sources
    # ClinicalTrials.gov
    ctgov_trials = await trial_searcher(
        conditions=all_conditions,
        other_terms=mutations,
        lat=location[0],
        long=location[1],
        distance=100,
        recruiting_status="OPEN"
    )

    # NCI (if API key available)
    if os.getenv("NCI_API_KEY"):
        nci_trials = await trial_searcher(
            source="nci",
            conditions=all_conditions,
            required_mutations=mutations,
            exclude_prior_therapy=prior_treatments,
            api_key=os.getenv("NCI_API_KEY")
        )

    return {
        "clinicaltrials_gov": ctgov_trials,
        "nci": nci_trials
    }

# Example usage
trials = await find_trials_for_patient(
    disease="melanoma",
    mutations=["BRAF V600E"],
    location=(40.7128, -74.0060),  # New York
    prior_treatments=["vemurafenib"]
)
```

### Workflow 2: Research Landscape Analysis

Understand ongoing research in a field:

```python
async def analyze_research_landscape(gene: str, disease: str):
    # Get gene information
    gene_info = await gene_getter(gene)

    # Find all active trials
    all_trials = await trial_searcher(
        conditions=[disease],
        other_terms=[gene, f"{gene} mutation", f"{gene} positive"],
        recruiting_status="OPEN",
        page_size=50
    )

    # Categorize by phase
    phase_distribution = {}
    for trial in all_trials:
        phase = trial.phase or "Not specified"
        phase_distribution[phase] = phase_distribution.get(phase, 0) + 1

    # Extract unique interventions
    interventions = set()
    for trial in all_trials:
        if trial.interventions:
            interventions.update(trial.interventions)

    return {
        "total_trials": len(all_trials),
        "phase_distribution": phase_distribution,
        "unique_interventions": list(interventions),
        "gene_info": gene_info
    }

# Example
landscape = await analyze_research_landscape("ALK", "lung cancer")
```

### Workflow 3: Biomarker-Driven Search

Find trials based on specific biomarkers:

```python
async def biomarker_trial_search(biomarkers: list[str], cancer_type: str):
    # Search NCI biomarker database
    biomarker_results = []
    for biomarker in biomarkers:
        result = await nci_biomarker_searcher(
            name=biomarker,
            api_key=os.getenv("NCI_API_KEY")
        )
        biomarker_results.extend(result)

    # Extract associated trials
    trial_ids = set()
    for bio in biomarker_results:
        if bio.get("associated_trials"):
            trial_ids.update(bio["associated_trials"])

    # Get trial details
    trials = []
    for nct_id in trial_ids:
        trial = await trial_getter(nct_id)
        trials.append(trial)

    return trials

# Example
trials = await biomarker_trial_search(
    biomarkers=["PD-L1", "TMB-high", "MSI-H"],
    cancer_type="colorectal cancer"
)
```

## Working with Trial Results

### Extracting Key Information

```python
# Process trial results
for trial in trials:
    print(f"NCT ID: {trial.nct_id}")
    print(f"Title: {trial.title}")
    print(f"Status: {trial.status}")
    print(f"Phase: {trial.phase}")

    # Locations
    if trial.locations:
        print("Locations:")
        for loc in trial.locations:
            print(f"  - {loc.facility}, {loc.city}, {loc.state}")

    # Eligibility
    if trial.eligibility:
        print(f"Age: {trial.eligibility.minimum_age} - {trial.eligibility.maximum_age}")
        print(f"Sex: {trial.eligibility.sex}")
```

### Getting Detailed Trial Information

```python
# Get complete trial details
full_trial = await trial_getter("NCT03006926")

# Get specific sections
protocol = await trial_protocol_getter("NCT03006926")
locations = await trial_locations_getter("NCT03006926")
outcomes = await trial_outcomes_getter("NCT03006926")
references = await trial_references_getter("NCT03006926")
```

## Tips for Effective Trial Searches

### 1. Use Multiple Search Terms

```python
# Cover variations
trials = await trial_searcher(
    conditions=["NSCLC", "non-small cell lung cancer", "lung adenocarcinoma"],
    interventions=["anti-PD-1", "pembrolizumab", "Keytruda"]
)
```

### 2. Check Both Data Sources

```python
# Some trials may only be in one database
ctgov_count = len(await trial_searcher(source="ctgov", conditions=["melanoma"]))
nci_count = len(await trial_searcher(source="nci", conditions=["melanoma"]))
```

### 3. Use Appropriate Filters

- **recruiting_status**: Focus on trials accepting patients
- **phase**: Later phases for established treatments
- **age_group**: Match patient demographics
- **study_type**: INTERVENTIONAL vs OBSERVATIONAL

### 4. Leverage Location Search

Always include location for patient-specific searches:

```python
# Bad - no location
trials = await trial_searcher(conditions=["cancer"])

# Good - includes location
trials = await trial_searcher(
    conditions=["cancer"],
    lat=40.7128,
    long=-74.0060,
    distance=50
)
```

## Troubleshooting

### No Results Found

1. **Broaden search terms**: Remove specific filters
2. **Check synonyms**: Use disease_getter to find alternatives
3. **Expand location**: Increase distance parameter
4. **Try both sources**: Some trials only in NCI or ClinicalTrials.gov

### Location Search Issues

- Ensure both latitude AND longitude are provided
- Use decimal degrees (not degrees/minutes/seconds)
- Check coordinate signs (negative for West/South)

### NCI API Errors

- Verify API key is valid
- Check rate limits (1000 requests/day with key)
- Some features require specific API key permissions

## Next Steps

- Learn about [variant annotations](03-get-comprehensive-variant-annotations.md)
- Explore [AlphaGenome predictions](04-predict-variant-effects-with-alphagenome.md)
- Set up [monitoring and logging](05-logging-and-monitoring-with-bigquery.md)
