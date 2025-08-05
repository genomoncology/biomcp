# NCI Clinical Trials Search API Reference

The National Cancer Institute's Clinical Trials Search (CTS) API provides advanced search capabilities for cancer clinical trials with enhanced filtering options beyond ClinicalTrials.gov.

## Overview

The NCI CTS API offers:

- Advanced biomarker and mutation filtering
- Comprehensive organization database
- Intervention and drug vocabularies
- Disease terminology with NCI Thesaurus integration
- Prior therapy and eligibility criteria

**Base URL:** `https://clinicaltrialsapi.cancer.gov/api/v2/`

## Authentication

An API key is required for all endpoints.

### Obtaining an API Key

1. Visit [https://clinicaltrialsapi.cancer.gov/](https://clinicaltrialsapi.cancer.gov/)
2. Click "Get API Key"
3. Complete registration
4. Key is emailed immediately

### Using the API Key

Include in request headers:

```
X-API-KEY: your-api-key-here
```

Or as query parameter:

```
?api_key=your-api-key-here
```

## Core Endpoints

### 1. Trial Search

```
GET /trials
```

Search for clinical trials with advanced filtering.

#### Parameters

**Basic Search:**

- `keyword`: General text search
- `nct_id`: Specific NCT identifiers
- `diseases`: Disease/condition names
- `interventions`: Treatment names

**Advanced Filters:**

- `biomarkers`: Required biomarkers/mutations
- `prior_therapy_required`: true/false
- `accepts_brain_mets`: true/false
- `min_age`: Minimum age in years
- `max_age`: Maximum age in years

**Pagination:**

- `size`: Results per page (max 50)
- `from`: Starting index (offset)

#### Example Request

```bash
curl -X GET "https://clinicaltrialsapi.cancer.gov/api/v2/trials" \
  -H "X-API-KEY: your-key" \
  -d "diseases=melanoma" \
  -d "biomarkers=BRAF V600E" \
  -d "accepts_brain_mets=true" \
  -d "size=10"
```

#### Response Format

```json
{
  "total": 42,
  "trials": [
    {
      "nct_id": "NCT04280705",
      "brief_title": "BRAF/MEK Inhibitor Combination",
      "current_trial_status": "Active",
      "phase": "Phase II",
      "biomarker_eligibility": [
        {
          "gene": "BRAF",
          "variant": "V600E",
          "required": true
        }
      ],
      "sites": [...]
    }
  ]
}
```

### 2. Trial Details

```
GET /trials/{nct_id}
```

Get comprehensive information about a specific trial.

#### Example Request

```bash
curl -X GET "https://clinicaltrialsapi.cancer.gov/api/v2/trials/NCT04280705" \
  -H "X-API-KEY: your-key"
```

### 3. Organization Search

```
GET /organizations
```

Search for cancer research organizations and treatment centers.

#### Parameters

- `name`: Organization name
- `org_city`: City location
- `org_state_or_province`: State/province
- `org_country`: Country
- `org_type`: Type (e.g., "NCI-designated", "academic")

**Important:** Always use city AND state together to avoid Elasticsearch errors.

#### Example Request

```bash
curl -X GET "https://clinicaltrialsapi.cancer.gov/api/v2/organizations" \
  -H "X-API-KEY: your-key" \
  -d "org_city=Houston" \
  -d "org_state_or_province=TX"
```

### 4. Organization Details

```
GET /organizations/{org_id}
```

Get details about a specific organization.

### 5. Intervention Search

```
GET /interventions
```

Search for drugs, devices, and procedures used in trials.

#### Parameters

- `name`: Intervention name
- `type`: Drug, Device, Procedure, etc.
- `synonyms`: Include synonym matches (default: true)

#### Example Request

```bash
curl -X GET "https://clinicaltrialsapi.cancer.gov/api/v2/interventions" \
  -H "X-API-KEY: your-key" \
  -d "name=pembrolizumab" \
  -d "type=Drug"
```

### 6. Intervention Details

```
GET /interventions/{intervention_id}
```

### 7. Biomarker Search

```
GET /biomarkers
```

Search for biomarkers used in trial eligibility criteria.

#### Parameters

- `name`: Biomarker name
- `type`: mutation, expression, etc.
- `gene`: Associated gene symbol

### 8. Disease Search

```
GET /diseases
```

Search NCI's controlled vocabulary of cancer conditions.

#### Parameters

- `name`: Disease name
- `include_synonyms`: Include synonym matches
- `category`: Disease category

## Advanced Features

### Biomarker-Based Trial Search

Find trials requiring specific mutations:

```python
params = {
    "diseases": "non-small cell lung cancer",
    "biomarkers": ["EGFR L858R", "EGFR exon 19 deletion"],
    "prior_therapy_required": False,
    "accepts_brain_mets": True
}

response = requests.get(
    "https://clinicaltrialsapi.cancer.gov/api/v2/trials",
    headers={"X-API-KEY": api_key},
    params=params
)
```

### Complex Eligibility Queries

```python
# Find trials with specific eligibility
params = {
    "diseases": "melanoma",
    "biomarkers": "BRAF V600E",
    "min_age": 18,
    "max_age": 75,
    "prior_therapy": "vemurafenib",  # Exclude if prior vemurafenib
    "performance_status": "0-1"       # ECOG 0 or 1
}
```

### Organization Network Analysis

```python
# Find all NCI-designated centers in a region
params = {
    "org_type": "NCI-designated",
    "org_state_or_province": ["CA", "OR", "WA"]  # West Coast
}

orgs = requests.get(
    "https://clinicaltrialsapi.cancer.gov/api/v2/organizations",
    headers={"X-API-KEY": api_key},
    params=params
)

# Get trials at each center
for org in orgs.json()["organizations"]:
    trials = requests.get(
        f"https://clinicaltrialsapi.cancer.gov/api/v2/trials",
        headers={"X-API-KEY": api_key},
        params={"site_org_id": org["id"]}
    )
```

## Data Models

### Trial Object

```json
{
  "nct_id": "NCT04280705",
  "brief_title": "Study Title",
  "official_title": "Full Protocol Title",
  "current_trial_status": "Active",
  "phase": "Phase II",
  "study_type": "Interventional",
  "primary_purpose": "Treatment",
  "diseases": [
    {
      "name": "Melanoma",
      "nci_thesaurus_id": "C0025202"
    }
  ],
  "biomarker_eligibility": [
    {
      "gene": "BRAF",
      "variant": "V600E",
      "required": true,
      "inclusion": true
    }
  ],
  "arms": [...],
  "sites": [...]
}
```

### Organization Object

```json
{
  "org_id": "NCI-2021-00123",
  "name": "MD Anderson Cancer Center",
  "type": "NCI-designated",
  "address": {
    "city": "Houston",
    "state": "TX",
    "country": "United States",
    "postal_code": "77030"
  },
  "contact": {
    "name": "Clinical Trials Office",
    "phone": "1-800-392-1611",
    "email": "clinical.trials@mdanderson.org"
  },
  "active_trials_count": 1250
}
```

## Error Handling

### Common Errors

#### 401 Unauthorized

```json
{
  "error": "Invalid or missing API key"
}
```

#### 400 Bad Request

```json
{
  "error": "Invalid parameter combination",
  "details": "Must specify both city AND state for location search"
}
```

#### 429 Rate Limited

```json
{
  "error": "Rate limit exceeded",
  "retry_after": 3600
}
```

### Best Practices

1. **Always use city AND state together** for location searches
2. **Handle missing totals** - the API may not return total counts with size parameter
3. **Use specific searches** - broad queries may timeout
4. **Implement retry logic** for rate limits

## Rate Limits

- **With API Key**: 1,000 requests/day
- **Burst Rate**: 10 requests/second
- **Without Key**: Not supported

## Differences from ClinicalTrials.gov

### Enhanced Features

- **Biomarker search**: Mutation-specific queries
- **Prior therapy**: Exclude based on previous treatments
- **Brain metastases**: Specific acceptance criteria
- **Performance status**: ECOG/Karnofsky filtering

### Limitations

- **Cancer trials only**: Limited to oncology studies
- **No offset pagination**: Must use size parameter carefully
- **Location parameters**: Different naming (org\_ prefix)

## Integration Examples

### Example 1: Precision Medicine Search

```python
async def find_precision_trials(mutation, cancer_type, location):
    """Find trials for specific mutation in cancer type near location"""

    # Search for trials
    trial_params = {
        "diseases": cancer_type,
        "biomarkers": mutation,
        "accepts_brain_mets": True,
        "size": 50
    }

    trials = await fetch_nci_api("trials", trial_params)

    # Filter by location if provided
    if location:
        nearby_trials = []
        for trial in trials["trials"]:
            for site in trial.get("sites", []):
                distance = calculate_distance(location, site["coordinates"])
                if distance < 100:  # 100 miles
                    nearby_trials.append(trial)
                    break

        return nearby_trials

    return trials["trials"]
```

### Example 2: Biomarker-Driven Pipeline

```python
def biomarker_trial_pipeline(gene, variant):
    """Complete pipeline from variant to trials"""

    # 1. Search biomarkers
    biomarkers = requests.get(
        "https://clinicaltrialsapi.cancer.gov/api/v2/biomarkers",
        headers={"X-API-KEY": api_key},
        params={"gene": gene, "name": variant}
    ).json()

    # 2. Get associated trials
    all_trials = []
    for biomarker in biomarkers.get("biomarkers", []):
        trials = requests.get(
            "https://clinicaltrialsapi.cancer.gov/api/v2/trials",
            headers={"X-API-KEY": api_key},
            params={"biomarker_id": biomarker["id"]}
        ).json()
        all_trials.extend(trials.get("trials", []))

    # 3. Deduplicate and sort by phase
    unique_trials = {t["nct_id"]: t for t in all_trials}.values()
    return sorted(unique_trials, key=lambda x: x.get("phase", ""))
```

## Support Resources

- **API Documentation**: [https://clinicaltrialsapi.cancer.gov/](https://clinicaltrialsapi.cancer.gov/)
- **Support Email**: NCICTSApiSupport@mail.nih.gov
- **Status Page**: [https://status.cancer.gov/](https://status.cancer.gov/)
- **Terms of Use**: [https://clinicaltrialsapi.cancer.gov/terms](https://clinicaltrialsapi.cancer.gov/terms)
