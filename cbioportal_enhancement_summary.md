# cBioPortal Enhancement Summary: Additional Mutation Data Fields

## Current Implementation
The current cBioPortal integration in BioMCP captures:
- **total_cases**: Total number of cases with the variant
- **studies**: List of study IDs containing the variant
- **tumor_types**: List placeholder (marked as "future use")

## Available Fields from cBioPortal API

Based on the API exploration, cBioPortal provides rich mutation data that we could extract:

### 1. **Mutation Type Classification** (HIGH VALUE)
- **Field**: `mutationType`
- **Values**: "Missense_Mutation", "Nonsense_Mutation", "Frame_Shift_Del", "Frame_Shift_Ins", "In_Frame_Del", "In_Frame_Ins", "Silent", "Splice_Site", etc.
- **Clinical Value**: Essential for understanding the functional impact of mutations
- **Implementation**: Already available in the mutation response

### 2. **Variant Allele Frequency (VAF)** (HIGH VALUE)
- **Fields**: `tumorAltCount`, `tumorRefCount`, `normalAltCount`, `normalRefCount`
- **Calculation**: VAF = tumorAltCount / (tumorAltCount + tumorRefCount)
- **Clinical Value**: Indicates clonality and tumor purity; helps distinguish driver vs passenger mutations
- **Implementation**: Already available in the mutation response

### 3. **Cancer Type Distribution** (HIGH VALUE)
- **Current Gap**: Currently only tracking study IDs, not cancer types
- **Solution**: Map study IDs to cancer types using study metadata
- **Clinical Value**: Shows which cancer types commonly have this mutation
- **Implementation**: Need to fetch study metadata and aggregate

### 4. **Mutation Keywords** (MEDIUM VALUE)
- **Field**: `keyword`
- **Examples**: "BRAF V600 missense", "BRAF truncating"
- **Clinical Value**: Pre-classified mutation categories for quick filtering
- **Implementation**: Already available in the mutation response

### 5. **Sample-Level Metadata** (MEDIUM VALUE)
- **Clinical Attributes Available**:
  - `SAMPLE_TYPE`: Primary vs Metastatic
  - `CANCER_TYPE_DETAILED`: Specific histological subtype
  - `TUMOR_STAGE`: Cancer stage information
- **Clinical Value**: Understand mutation prevalence in different disease contexts
- **Implementation**: Requires additional clinical data API calls

### 6. **Co-occurring Mutations** (HIGH VALUE - Future Enhancement)
- **Approach**: Query all mutations for samples with the target mutation
- **Clinical Value**: Identify mutation signatures and therapeutic vulnerabilities
- **Implementation**: Complex - requires multiple API calls per sample

### 7. **Clinical Outcomes** (HIGH VALUE - Limited Availability)
- **Available Fields**: 
  - Overall survival (OS_STATUS, OS_MONTHS)
  - Disease-free survival (DFS_STATUS, DFS_MONTHS)
- **Clinical Value**: Direct correlation with patient outcomes
- **Implementation**: Requires patient-level clinical data queries
- **Limitation**: Not all studies have outcome data

## Recommended Enhancements Priority

### Phase 1 - Immediate Enhancements
1. **Mutation Type Classification**
   - Add `mutation_type` field to CBioPortalVariantData
   - Extract from existing API response

2. **Variant Allele Frequency**
   - Add `vaf` field (calculated)
   - Add raw counts for transparency

3. **Cancer Type Distribution**
   - Properly populate the existing `tumor_types` field
   - Create cancer type → count mapping

### Phase 2 - Medium Priority
4. **Sample Type Distribution**
   - Add `primary_count` and `metastatic_count` fields
   - Requires clinical data API integration

5. **Mutation Keywords**
   - Add `keywords` field for pre-classified categories

### Phase 3 - Complex Features
6. **Co-occurring Mutations**
   - Create separate endpoint/method
   - Add `common_co_mutations` field with top 5-10

7. **Clinical Outcomes**
   - Add optional `survival_data` nested object
   - Only populate when available

## Implementation Recommendations

1. **Minimize API Calls**: Batch requests where possible to avoid rate limiting
2. **Caching**: Consider caching study metadata since it changes infrequently  
3. **Progressive Enhancement**: Return basic data quickly, enrich with additional fields asynchronously
4. **Error Handling**: Gracefully handle missing fields since not all studies have all data types
5. **Documentation**: Clearly document which fields may be null/empty depending on the study

## Example Enhanced Response Structure

```python
class CBioPortalVariantData(BaseModel):
    """Enhanced cBioPortal variant annotation data."""
    
    # Current fields
    total_cases: int | None
    studies: list[str]
    
    # Phase 1 additions
    mutation_types: dict[str, int]  # {"Missense_Mutation": 45, "Nonsense_Mutation": 3}
    cancer_types: dict[str, int]    # {"Breast Cancer": 20, "Lung Cancer": 15}
    vaf_stats: dict[str, float]     # {"mean": 0.35, "median": 0.30, "max": 0.85}
    
    # Phase 2 additions
    sample_types: dict[str, int]    # {"primary": 40, "metastatic": 25}
    keywords: list[str]             # ["BRAF V600 missense", "BRAF hotspot"]
    
    # Phase 3 additions (optional)
    co_mutations: list[dict]        # [{"gene": "TP53", "frequency": 0.45}, ...]
    survival_impact: dict | None    # {"median_os_months": 24.5, "hazard_ratio": 1.8}
```

This enhancement would transform the cBioPortal integration from a simple occurrence counter to a comprehensive clinical annotation tool that provides actionable insights for researchers and clinicians.