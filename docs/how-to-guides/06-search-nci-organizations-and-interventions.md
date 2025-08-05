# How to Search NCI Organizations and Interventions

This guide demonstrates how to use BioMCP's NCI-specific tools to search for cancer research organizations, interventions (drugs, devices, procedures), and biomarkers.

## Prerequisites

All NCI tools require an API key from [api.cancer.gov](https://api.cancer.gov):

```bash
# Set as environment variable
export NCI_API_KEY="your-key-here"

# Or provide per-request in your prompts
"Find cancer centers in Boston, my NCI API key is YOUR_KEY"
```

## Organization Search and Lookup

### Understanding Organization Search

The NCI Organization database contains:

- Cancer research centers and hospitals
- Clinical trial sponsors
- Academic institutions
- Pharmaceutical companies
- Government facilities

### Basic Organization Search

Find organizations by name:

```bash
# CLI
biomcp organization search --name "MD Anderson" --api-key YOUR_KEY

# Python
orgs = await nci_organization_searcher(
    name="MD Anderson",
    api_key="your-key"
)

# MCP/AI Assistant
"Search for MD Anderson Cancer Center, my NCI API key is YOUR_KEY"
```

### Location-Based Search

**CRITICAL**: Always use city AND state together to avoid Elasticsearch errors!

```python
# ✅ CORRECT - City and state together
orgs = await nci_organization_searcher(
    city="Houston",
    state="TX",
    api_key="your-key"
)

# ❌ WRONG - Will cause API error
orgs = await nci_organization_searcher(
    city="Houston",  # Missing state!
    api_key="your-key"
)

# ❌ WRONG - Will cause API error
orgs = await nci_organization_searcher(
    state="TX",  # Missing city!
    api_key="your-key"
)
```

### Organization Types

Search by organization type:

```python
# Find academic cancer centers
academic_centers = await nci_organization_searcher(
    organization_type="Academic",
    api_key="your-key"
)

# Find pharmaceutical companies
pharma_companies = await nci_organization_searcher(
    organization_type="Industry",
    api_key="your-key"
)

# Find government research facilities
gov_facilities = await nci_organization_searcher(
    organization_type="Government",
    api_key="your-key"
)
```

Valid organization types:

- `Academic` - Universities and medical schools
- `Industry` - Pharmaceutical and biotech companies
- `Government` - NIH, FDA, VA hospitals
- `Community` - Community hospitals and clinics
- `Network` - Research networks and consortiums
- `Other` - Other organization types

### Getting Organization Details

Retrieve complete information about a specific organization:

```python
# Get organization by ID
org_details = await nci_organization_getter(
    organization_id="NCI-2011-03337",
    api_key="your-key"
)

# Returns:
# - Full name and aliases
# - Contact information
# - Address and location
# - Associated clinical trials
# - Organization type and status
```

### Practical Organization Workflows

#### Find Regional Cancer Centers

```python
async def find_cancer_centers_by_region(state: str, cities: list[str]):
    """Find all cancer centers in specific cities within a state"""

    all_centers = []

    for city in cities:
        # ALWAYS use city + state together
        centers = await nci_organization_searcher(
            city=city,
            state=state,
            organization_type="Academic",
            api_key=os.getenv("NCI_API_KEY")
        )
        all_centers.extend(centers)

    # Remove duplicates
    unique_centers = {org['id']: org for org in all_centers}

    return list(unique_centers.values())

# Example: Find cancer centers in major Texas cities
texas_centers = await find_cancer_centers_by_region(
    state="TX",
    cities=["Houston", "Dallas", "San Antonio", "Austin"]
)
```

#### Find Trial Sponsors

```python
async def find_trial_sponsors_by_type(org_type: str, name_filter: str = None):
    """Find organizations sponsoring trials"""

    # Search organizations
    orgs = await nci_organization_searcher(
        name=name_filter,
        organization_type=org_type,
        api_key=os.getenv("NCI_API_KEY")
    )

    # For each org, get details including trial count
    sponsors = []
    for org in orgs[:10]:  # Limit to avoid rate limits
        details = await nci_organization_getter(
            organization_id=org['id'],
            api_key=os.getenv("NCI_API_KEY")
        )
        if details.get('trial_count', 0) > 0:
            sponsors.append(details)

    return sorted(sponsors, key=lambda x: x.get('trial_count', 0), reverse=True)

# Find pharmaceutical companies with active trials
pharma_sponsors = await find_trial_sponsors_by_type("Industry")
```

## Intervention Search and Lookup

### Understanding Interventions

Interventions in clinical trials include:

- **Drugs**: Chemotherapy, targeted therapy, immunotherapy
- **Devices**: Medical devices, diagnostic tools
- **Procedures**: Surgical techniques, radiation protocols
- **Biologicals**: Cell therapies, vaccines, antibodies
- **Behavioral**: Lifestyle interventions, counseling
- **Other**: Dietary supplements, alternative therapies

### Drug Search

Find specific drugs or drug classes:

```bash
# CLI - Find a specific drug
biomcp intervention search --name pembrolizumab --type Drug --api-key YOUR_KEY

# CLI - Find drug class
biomcp intervention search --name "PD-1 inhibitor" --type Drug --api-key YOUR_KEY
```

```python
# Python - Search with synonyms
drugs = await nci_intervention_searcher(
    name="pembrolizumab",
    intervention_type="Drug",
    synonyms=True,  # Include Keytruda, MK-3475, etc.
    api_key="your-key"
)

# Search for drug combinations
combos = await nci_intervention_searcher(
    name="nivolumab AND ipilimumab",
    intervention_type="Drug",
    api_key="your-key"
)
```

### Device and Procedure Search

```python
# Find medical devices
devices = await nci_intervention_searcher(
    intervention_type="Device",
    name="robot",  # Surgical robots
    api_key="your-key"
)

# Find procedures
procedures = await nci_intervention_searcher(
    intervention_type="Procedure",
    name="minimally invasive",
    api_key="your-key"
)

# Find radiation protocols
radiation = await nci_intervention_searcher(
    intervention_type="Radiation",
    name="proton beam",
    api_key="your-key"
)
```

### Getting Intervention Details

```python
# Get complete intervention information
intervention = await nci_intervention_getter(
    intervention_id="INT123456",
    api_key="your-key"
)

# Returns:
# - Official name and synonyms
# - Intervention type and subtype
# - Mechanism of action (for drugs)
# - FDA approval status
# - Associated clinical trials
# - Manufacturer information
```

### Practical Intervention Workflows

#### Drug Development Pipeline

```python
async def analyze_drug_pipeline(drug_target: str):
    """Analyze drugs in development for a specific target"""

    # Search for drugs targeting specific pathway
    drugs = await nci_intervention_searcher(
        name=drug_target,
        intervention_type="Drug",
        api_key=os.getenv("NCI_API_KEY")
    )

    pipeline = {
        "preclinical": [],
        "phase1": [],
        "phase2": [],
        "phase3": [],
        "approved": []
    }

    for drug in drugs:
        # Get detailed information
        details = await nci_intervention_getter(
            intervention_id=drug['id'],
            api_key=os.getenv("NCI_API_KEY")
        )

        # Categorize by development stage
        if details.get('fda_approved'):
            pipeline['approved'].append(details)
        else:
            # Check associated trials for phase
            trial_phases = details.get('trial_phases', [])
            if 'PHASE3' in trial_phases:
                pipeline['phase3'].append(details)
            elif 'PHASE2' in trial_phases:
                pipeline['phase2'].append(details)
            elif 'PHASE1' in trial_phases:
                pipeline['phase1'].append(details)
            else:
                pipeline['preclinical'].append(details)

    return pipeline

# Analyze PD-1/PD-L1 inhibitor pipeline
pd1_pipeline = await analyze_drug_pipeline("PD-1 inhibitor")
```

#### Compare Similar Interventions

```python
async def compare_interventions(intervention_names: list[str]):
    """Compare multiple interventions side by side"""

    comparisons = []

    for name in intervention_names:
        # Search for intervention
        results = await nci_intervention_searcher(
            name=name,
            synonyms=True,
            api_key=os.getenv("NCI_API_KEY")
        )

        if results:
            # Get detailed info for first match
            details = await nci_intervention_getter(
                intervention_id=results[0]['id'],
                api_key=os.getenv("NCI_API_KEY")
            )

            comparisons.append({
                "name": details['name'],
                "type": details['type'],
                "synonyms": details.get('synonyms', []),
                "fda_approved": details.get('fda_approved', False),
                "trial_count": len(details.get('trials', [])),
                "mechanism": details.get('mechanism_of_action', 'Not specified')
            })

    return comparisons

# Compare checkpoint inhibitors
comparison = await compare_interventions([
    "pembrolizumab",
    "nivolumab",
    "atezolizumab",
    "durvalumab"
])
```

## Biomarker Search

### Understanding Biomarker Types

The NCI API supports two biomarker types:

- `reference_gene` - Gene-based biomarkers (e.g., EGFR, BRAF)
- `branch` - Pathway/branch biomarkers

**Note**: You cannot search by gene symbol directly; use the name parameter.

### Basic Biomarker Search

```python
# Search for PD-L1 biomarkers
pdl1_biomarkers = await nci_biomarker_searcher(
    name="PD-L1",
    api_key="your-key"
)

# Search for specific biomarker type
gene_biomarkers = await nci_biomarker_searcher(
    biomarker_type="reference_gene",
    api_key="your-key"
)
```

### Biomarker Analysis Workflow

```python
async def analyze_trial_biomarkers(disease: str):
    """Find biomarkers used in trials for a disease"""

    # Get all biomarkers
    all_biomarkers = await nci_biomarker_searcher(
        biomarker_type="reference_gene",
        api_key=os.getenv("NCI_API_KEY")
    )

    # Filter by disease association
    disease_biomarkers = []
    for biomarker in all_biomarkers:
        if disease.lower() in str(biomarker).lower():
            disease_biomarkers.append(biomarker)

    # Group by frequency
    biomarker_counts = {}
    for bio in disease_biomarkers:
        name = bio.get('name', 'Unknown')
        biomarker_counts[name] = biomarker_counts.get(name, 0) + 1

    # Sort by frequency
    return sorted(
        biomarker_counts.items(),
        key=lambda x: x[1],
        reverse=True
    )

# Find most common biomarkers in lung cancer trials
lung_biomarkers = await analyze_trial_biomarkers("lung cancer")
```

## Combined Workflows

### Regional Drug Development Analysis

```python
async def analyze_regional_drug_development(
    state: str,
    cities: list[str],
    drug_class: str
):
    """Analyze drug development in a specific region"""

    # Step 1: Find organizations in the region
    organizations = []
    for city in cities:
        orgs = await nci_organization_searcher(
            city=city,
            state=state,
            organization_type="Industry",
            api_key=os.getenv("NCI_API_KEY")
        )
        organizations.extend(orgs)

    # Step 2: Find drugs of interest
    drugs = await nci_intervention_searcher(
        name=drug_class,
        intervention_type="Drug",
        api_key=os.getenv("NCI_API_KEY")
    )

    # Step 3: Cross-reference trials
    regional_development = []
    for drug in drugs[:10]:  # Limit for performance
        drug_details = await nci_intervention_getter(
            intervention_id=drug['id'],
            api_key=os.getenv("NCI_API_KEY")
        )

        # Check if any trials are sponsored by regional orgs
        for trial in drug_details.get('trials', []):
            for org in organizations:
                if org['id'] in str(trial):
                    regional_development.append({
                        'drug': drug_details['name'],
                        'organization': org['name'],
                        'location': f"{org.get('city', '')}, {org.get('state', '')}",
                        'trial': trial
                    })

    return regional_development

# Analyze immunotherapy development in California
ca_immuno = await analyze_regional_drug_development(
    state="CA",
    cities=["San Francisco", "San Diego", "Los Angeles"],
    drug_class="immunotherapy"
)
```

### Organization to Intervention Pipeline

```python
async def org_to_intervention_pipeline(org_name: str):
    """Trace from organization to their interventions"""

    # Find organization
    orgs = await nci_organization_searcher(
        name=org_name,
        api_key=os.getenv("NCI_API_KEY")
    )

    if not orgs:
        return None

    # Get organization details
    org_details = await nci_organization_getter(
        organization_id=orgs[0]['id'],
        api_key=os.getenv("NCI_API_KEY")
    )

    # Get their trials
    org_trials = org_details.get('trials', [])

    # Extract unique interventions
    interventions = set()
    for trial_id in org_trials[:20]:  # Sample trials
        trial = await trial_getter(
            nct_id=trial_id,
            source="nci",
            api_key=os.getenv("NCI_API_KEY")
        )

        if trial.get('interventions'):
            interventions.update(trial['interventions'])

    # Get details for each intervention
    intervention_details = []
    for intervention_name in interventions:
        results = await nci_intervention_searcher(
            name=intervention_name,
            api_key=os.getenv("NCI_API_KEY")
        )
        if results:
            intervention_details.append(results[0])

    return {
        'organization': org_details,
        'trial_count': len(org_trials),
        'interventions': intervention_details
    }

# Analyze Genentech's intervention portfolio
genentech_portfolio = await org_to_intervention_pipeline("Genentech")
```

## Best Practices

### 1. Always Use City + State Together

```python
# ✅ GOOD - Prevents API errors
await nci_organization_searcher(city="Boston", state="MA")

# ❌ BAD - Will cause Elasticsearch error
await nci_organization_searcher(city="Boston")
```

### 2. Handle Rate Limits

```python
import asyncio

async def search_with_rate_limit(searches: list):
    """Execute searches with rate limiting"""
    results = []

    for search in searches:
        result = await search()
        results.append(result)

        # Add delay to respect rate limits
        await asyncio.sleep(0.1)  # 10 requests per second

    return results
```

### 3. Use Pagination for Large Results

```python
async def get_all_organizations(org_type: str):
    """Get all organizations of a type using pagination"""

    all_orgs = []
    page = 1

    while True:
        orgs = await nci_organization_searcher(
            organization_type=org_type,
            page=page,
            page_size=100,  # Maximum allowed
            api_key=os.getenv("NCI_API_KEY")
        )

        if not orgs:
            break

        all_orgs.extend(orgs)
        page += 1

        # Note: Total count may not be available
        if len(orgs) < 100:
            break

    return all_orgs
```

### 4. Cache Results

```python
from functools import lru_cache
import hashlib

@lru_cache(maxsize=100)
async def cached_org_search(city: str, state: str, org_type: str):
    """Cache organization searches to reduce API calls"""

    return await nci_organization_searcher(
        city=city,
        state=state,
        organization_type=org_type,
        api_key=os.getenv("NCI_API_KEY")
    )
```

## Troubleshooting

### Common Errors and Solutions

1. **"Search Too Broad" Error**

   - Always use city + state together for location searches
   - Add more specific filters (name, type)
   - Reduce page_size parameter

2. **"NCI API key required"**

   - Set NCI_API_KEY environment variable
   - Or provide api_key parameter in function calls
   - Or include in prompt: "my NCI API key is YOUR_KEY"

3. **No Results Found**

   - Check spelling of organization/drug names
   - Try partial name matches
   - Remove filters and broaden search
   - Enable synonyms for intervention searches

4. **Rate Limit Exceeded**
   - Add delays between requests
   - Reduce concurrent requests
   - Cache frequently accessed data
   - Consider upgrading API key tier

### Debugging Tips

```python
# Enable debug logging
import logging
logging.basicConfig(level=logging.DEBUG)

# Test API key
async def test_nci_connection():
    try:
        result = await nci_organization_searcher(
            name="Mayo",
            api_key=os.getenv("NCI_API_KEY")
        )
        print(f"✅ API key valid, found {len(result)} results")
    except Exception as e:
        print(f"❌ API key error: {e}")

# Check specific organization exists
async def verify_org_id(org_id: str):
    try:
        org = await nci_organization_getter(
            organization_id=org_id,
            api_key=os.getenv("NCI_API_KEY")
        )
        print(f"✅ Organization found: {org['name']}")
    except:
        print(f"❌ Organization ID not found: {org_id}")
```

## Next Steps

- Review [NCI prompts examples](../tutorials/nci-prompts.md) for AI assistant usage
- Explore [trial search with biomarkers](02-find-trials-with-nci-and-biothings.md)
- Learn about [variant effect prediction](04-predict-variant-effects-with-alphagenome.md)
- Set up [API authentication](../getting-started/03-authentication-and-api-keys.md)
