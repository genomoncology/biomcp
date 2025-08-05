# NCI Tools Example Prompts

This guide provides example prompts for AI assistants to effectively use the NCI (National Cancer Institute) Clinical Trials Search API tools in BioMCP.

## Overview of NCI Tools

BioMCP integrates with the NCI Clinical Trials Search API to provide:

- **Organization Search & Lookup** - Find cancer research centers, hospitals, and trial sponsors
- **Intervention Search & Lookup** - Search for drugs, devices, procedures, and other interventions

These tools require an NCI API key from: https://clinicaltrialsapi.cancer.gov/

## Best Practices

### API Key Required

All example prompts in this guide should include your NCI API key. Add this to the end of each prompt:

```
"... my NCI API key is YOUR_API_KEY"
```

### Location Searches

**ALWAYS use city AND state together** when searching organizations by location. The NCI API has Elasticsearch limitations that cause errors with broad searches.

✅ **Good**: `nci_organization_searcher(city="Cleveland", state="OH")`
❌ **Bad**: `nci_organization_searcher(city="Cleveland")` or `nci_organization_searcher(state="OH")`

### API Parameter Notes

- The NCI APIs do not support offset-based pagination (`from` parameter)
- Organization location parameters use `org_` prefix (e.g., `org_city`, `org_state_or_province`)
- When using `size` parameter, the API may not return a `total` count

### Avoiding API Errors

- Use specific organization names when possible
- Combine multiple filters (name + type, city + state)
- Start with more specific searches, then broaden if needed

## Organization Tools

### Organization Search

#### Basic Organization Search

```
"Find cancer centers in California, my NCI API key is YOUR_API_KEY"
"Search for MD Anderson Cancer Center, my NCI API key is YOUR_API_KEY"
"List academic cancer research centers in New York, my NCI API key is YOUR_API_KEY"
"Find all NCI-designated cancer centers, my NCI API key is YOUR_API_KEY"
```

**Expected tool usage**: `nci_organization_searcher(state="CA", organization_type="Academic")`

#### Organization by Location

**IMPORTANT**: Always use city AND state together to avoid API errors!

```
"Show me cancer treatment centers in Boston, MA, my NCI API key is YOUR_API_KEY"
"Find clinical trial sites in Houston, Texas, my NCI API key is YOUR_API_KEY"
"List all cancer research organizations in Cleveland, OH, my NCI API key is YOUR_API_KEY"
"Search for industry sponsors in San Francisco, CA, my NCI API key is YOUR_API_KEY"
```

**Expected tool usage**: `nci_organization_searcher(city="Boston", state="MA")` ✓
**Never use**: `nci_organization_searcher(city="Boston")` ✗ or `nci_organization_searcher(state="MA")` ✗

#### Organization by Type

```
"Find all government cancer research facilities, my NCI API key is YOUR_API_KEY"
"List pharmaceutical companies running cancer trials, my NCI API key is YOUR_API_KEY"
"Show me academic medical centers conducting trials, my NCI API key is YOUR_API_KEY"
"Find community hospitals participating in cancer research, my NCI API key is YOUR_API_KEY"
```

**Expected tool usage**: `nci_organization_searcher(organization_type="Industry")`

### Organization Details

```
"Get details about organization NCI-2011-03337, my NCI API key is YOUR_API_KEY"
"Show me contact information for this cancer center, my NCI API key is YOUR_API_KEY"
"What trials is this organization conducting? My NCI API key is YOUR_API_KEY"
"Give me the full profile of this research institution, my NCI API key is YOUR_API_KEY"
```

**Expected tool usage**: `organization_getter(organization_id="NCI-2011-03337")`

## Intervention Tools

### Intervention Search

#### Drug Search

```
"Find all trials using pembrolizumab, my NCI API key is YOUR_API_KEY"
"Search for PD-1 inhibitor drugs in trials, my NCI API key is YOUR_API_KEY"
"List all immunotherapy drugs being tested, my NCI API key is YOUR_API_KEY"
"Find trials using Keytruda or similar drugs, my NCI API key is YOUR_API_KEY"
```

**Expected tool usage**: `nci_intervention_searcher(name="pembrolizumab", intervention_type="Drug")`

#### Device Search

```
"Search for medical devices in cancer trials, my NCI API key is YOUR_API_KEY"
"Find trials using surgical robots, my NCI API key is YOUR_API_KEY"
"List radiation therapy devices being tested, my NCI API key is YOUR_API_KEY"
"Show me trials with diagnostic devices, my NCI API key is YOUR_API_KEY"
```

**Expected tool usage**: `nci_intervention_searcher(intervention_type="Device")`

#### Procedure Search

```
"Find surgical procedures in cancer trials, my NCI API key is YOUR_API_KEY"
"Search for minimally invasive surgery trials, my NCI API key is YOUR_API_KEY"
"List trials with radiation therapy procedures, my NCI API key is YOUR_API_KEY"
"Show me trials testing new biopsy techniques, my NCI API key is YOUR_API_KEY"
```

**Expected tool usage**: `nci_intervention_searcher(intervention_type="Procedure")`

#### Other Interventions

```
"Find behavioral interventions for cancer patients, my NCI API key is YOUR_API_KEY"
"Search for dietary interventions in trials, my NCI API key is YOUR_API_KEY"
"List genetic therapy trials, my NCI API key is YOUR_API_KEY"
"Show me trials with exercise interventions, my NCI API key is YOUR_API_KEY"
```

**Expected tool usage**: `nci_intervention_searcher(intervention_type="Behavioral")`

### Intervention Details

```
"Get full details about intervention INT123456, my NCI API key is YOUR_API_KEY"
"Show me the mechanism of action for this drug, my NCI API key is YOUR_API_KEY"
"Is this intervention FDA approved? My NCI API key is YOUR_API_KEY"
"What trials are using this intervention? My NCI API key is YOUR_API_KEY"
```

**Expected tool usage**: `intervention_getter(intervention_id="INT123456")`

## Biomarker Tools

### Biomarker Search

#### Basic Biomarker Search

```
"Find PD-L1 expression biomarkers, my NCI API key is YOUR_API_KEY"
"Search for EGFR mutations used in trials, my NCI API key is YOUR_API_KEY"
"List biomarkers tested by IHC, my NCI API key is YOUR_API_KEY"
"Find HER2 positive biomarkers, my NCI API key is YOUR_API_KEY"
```

**Expected tool usage**: `nci_biomarker_searcher(name="PD-L1")`

#### Biomarker by Type

```
"Show me all reference gene biomarkers, my NCI API key is YOUR_API_KEY"
"Find branch biomarkers, my NCI API key is YOUR_API_KEY"
"List all biomarkers of type reference_gene, my NCI API key is YOUR_API_KEY"
```

**Expected tool usage**: `nci_biomarker_searcher(biomarker_type="reference_gene")`

#### Important Note on Biomarker Types

The NCI API only supports two biomarker types:

- `reference_gene`: Gene-based biomarkers
- `branch`: Branch/pathway biomarkers

Note: The API does NOT support searching by gene symbol or assay type directly.

## NCI Disease Tools

### Disease Search

#### Basic Disease Search

```
"Find melanoma in NCI vocabulary, my NCI API key is YOUR_API_KEY"
"Search for lung cancer types, my NCI API key is YOUR_API_KEY"
"List breast cancer subtypes, my NCI API key is YOUR_API_KEY"
"Find official name for GIST, my NCI API key is YOUR_API_KEY"
```

**Expected tool usage**: `nci_disease_searcher(name="melanoma")`

#### Disease with Synonyms

```
"Find all names for gastrointestinal stromal tumor, my NCI API key is YOUR_API_KEY"
"Search for NSCLC and all its synonyms, my NCI API key is YOUR_API_KEY"
"List all terms for triple-negative breast cancer, my NCI API key is YOUR_API_KEY"
"Find alternative names for melanoma, my NCI API key is YOUR_API_KEY"
```

**Expected tool usage**: `nci_disease_searcher(name="GIST", include_synonyms=True)`

## Combined Workflows

### Finding Trials at Specific Centers

```
"First find cancer centers in California, then show me their trials, my NCI API key is YOUR_API_KEY"
```

**Expected workflow**:

1. `nci_organization_searcher(state="CA")`
2. For each organization, search trials with that sponsor

### Drug Development Pipeline

```
"Search for CAR-T cell therapies and show me which organizations are developing them, my NCI API key is YOUR_API_KEY"
```

**Expected workflow**:

1. `nci_intervention_searcher(name="CAR-T", intervention_type="Biological")`
2. For each intervention, get details to see associated trials
3. Extract organization information from trial data

### Regional Cancer Research

```
"What cancer drugs are being tested in Boston area hospitals? My NCI API key is YOUR_API_KEY"
```

**Expected workflow**:

1. `nci_organization_searcher(city="Boston", state="MA")`
2. `trial_searcher(location="Boston, MA", source="nci")` with organization filters
3. Extract intervention information from trials

## Important Notes

### API Key Handling

All NCI tools require an API key. The tools will check for:

1. API key provided in the function call
2. `NCI_API_KEY` environment variable
3. User-provided key in their message (e.g., "my NCI API key is...")

### Synonym Support

The intervention searcher includes a `synonyms` parameter (default: True) that will search for:

- Drug trade names (e.g., "Keytruda" finds "pembrolizumab")
- Alternative spellings
- Related terms

### Pagination

Both search tools support pagination:

- `page`: Page number (1-based)
- `page_size`: Results per page (max 100)

### Organization Types

Valid organization types include:

- Academic
- Industry
- Government
- Community
- Network
- Other

### Intervention Types

Valid intervention types include:

- Drug
- Device
- Biological
- Procedure
- Radiation
- Behavioral
- Genetic
- Dietary
- Other

## Error Handling

Common errors and solutions:

1. **"NCI API key required"**: User needs to provide an API key
2. **"No results found"**: Try broader search terms or remove filters
3. **"Invalid organization/intervention ID"**: Verify the ID format
4. **Rate limiting**: The API has rate limits; wait before retrying
5. **"Search Too Broad" (Elasticsearch error)**: The search returns too many results
   - This happens when searching with broad criteria
   - **Prevention**: Always use city AND state together for location searches
   - Add organization name (even partial) to narrow results
   - Avoid searching by state alone or organization type alone
