# OpenFDA Example Prompts for AI Agents

This document provides example prompts that demonstrate effective use of BioMCP's OpenFDA integration for various precision oncology use cases.

## Drug Safety Assessment

### Basic Safety Profile

```
What are the most common adverse events reported for pembrolizumab?
Include both serious and non-serious events.
```

**Expected BioMCP Usage:**

1. `think` - Plan safety assessment approach
2. `openfda_adverse_searcher(drug="pembrolizumab", limit=50)`
3. Analyze and summarize top reactions

### Comparative Safety Analysis

```
Compare the adverse event profiles of imatinib and dasatinib for CML treatment.
Focus on serious events and their frequencies.
```

**Expected BioMCP Usage:**

1. `think` - Plan comparative analysis
2. `openfda_adverse_searcher(drug="imatinib", serious=True)`
3. `openfda_adverse_searcher(drug="dasatinib", serious=True)`
4. Compare and contrast findings

### Drug Interaction Investigation

```
A patient on warfarin needs to start erlotinib for NSCLC. What drug interactions
and adverse events should we monitor based on FDA data?
```

**Expected BioMCP Usage:**

1. `think` - Consider interaction risks
2. `openfda_label_searcher(name="erlotinib")` - Check drug interactions section
3. `openfda_adverse_searcher(drug="erlotinib", reaction="bleeding")`
4. `openfda_adverse_searcher(drug="erlotinib", reaction="INR")`

## Treatment Planning

### Indication Verification

```
Is trastuzumab deruxtecan FDA-approved for HER2-low breast cancer?
What are the specific approved indications?
```

**Expected BioMCP Usage:**

1. `think` - Plan indication search
2. `openfda_label_searcher(name="trastuzumab deruxtecan")`
3. `openfda_label_getter(set_id="...")` - Get full indications section
4. Extract and summarize approved uses

### Contraindication Screening

```
Patient has severe hepatic impairment. Which targeted therapy drugs for
melanoma have contraindications or warnings for liver dysfunction?
```

**Expected BioMCP Usage:**

1. `think` - Identify melanoma drugs to check
2. `openfda_label_searcher(indication="melanoma")`
3. For each drug: `openfda_label_getter(set_id="...", sections=["contraindications", "warnings_and_precautions"])`
4. Summarize liver-related contraindications

### Dosing Guidelines

```
What is the FDA-recommended dosing for osimertinib in EGFR-mutated NSCLC,
including dose modifications for adverse events?
```

**Expected BioMCP Usage:**

1. `think` - Plan dosing information retrieval
2. `openfda_label_searcher(name="osimertinib")`
3. `openfda_label_getter(set_id="...", sections=["dosage_and_administration", "dose_modifications"])`
4. Extract dosing guidelines

## Device Reliability Assessment

### Genomic Test Reliability

```
What adverse events have been reported for NGS-based cancer diagnostic devices?
Show me any false positive or accuracy issues.
```

**Expected BioMCP Usage:**

1. `think` - Consider test reliability factors
2. `openfda_device_searcher(genomics_only=True, limit=25)` - Get all genomic device events
3. `openfda_device_searcher(problem="false positive", genomics_only=True)`
4. `openfda_device_searcher(problem="accuracy", genomics_only=True)`
5. For significant events: `openfda_device_getter(mdr_report_key="...")`

**Note:** The FDA database uses abbreviated names (e.g., "F1CDX" instead of "FoundationOne CDx").
For specific devices, try: `openfda_device_searcher(device="F1CDX")` or search by key terms.

### Laboratory Equipment Issues

```
Our lab uses Illumina sequencers. What device malfunctions have been
reported that could impact our genomic testing workflow?
```

**Expected BioMCP Usage:**

1. `think` - Assess potential workflow impacts
2. `openfda_device_searcher(manufacturer="Illumina", genomics_only=True)`
3. Analyze problem patterns
4. `openfda_device_getter(mdr_report_key="...")` for critical issues

## Comprehensive Drug Evaluation

### New Drug Assessment

```
Provide a comprehensive safety and efficacy profile for sotorasib (Lumakras)
including FDA approval, indications, major warnings, and post-market adverse events.
```

**Expected BioMCP Usage:**

1. `think` - Plan comprehensive assessment
2. `drug_getter("sotorasib")` - Basic drug info
3. `openfda_label_searcher(name="sotorasib")`
4. `openfda_label_getter(set_id="...")` - Full label
5. `openfda_adverse_searcher(drug="sotorasib", serious=True)`
6. `trial_searcher(interventions=["sotorasib"])` - Ongoing trials

### Risk-Benefit Analysis

```
For a 75-year-old patient with metastatic melanoma, analyze the risk-benefit
profile of nivolumab plus ipilimumab combination therapy based on FDA data.
```

**Expected BioMCP Usage:**

1. `think` - Structure risk-benefit analysis
2. `openfda_label_searcher(name="nivolumab")`
3. `openfda_label_searcher(name="ipilimumab")`
4. `openfda_label_getter(set_id="...", sections=["geriatric_use", "warnings_and_precautions"])`
5. `openfda_adverse_searcher(drug="nivolumab", serious=True)`
6. `openfda_adverse_searcher(drug="ipilimumab", serious=True)`

## Special Populations

### Pregnancy Considerations

```
Which FDA-approved lung cancer treatments have pregnancy category data
or specific warnings for pregnant patients?
```

**Expected BioMCP Usage:**

1. `think` - Plan pregnancy safety search
2. `openfda_label_searcher(indication="lung cancer")`
3. For each drug: `openfda_label_getter(set_id="...", sections=["pregnancy", "use_in_specific_populations"])`
4. Compile pregnancy categories and warnings

### Pediatric Oncology

```
What FDA-approved indications and safety data exist for using
checkpoint inhibitors in pediatric cancer patients?
```

**Expected BioMCP Usage:**

1. `think` - Identify checkpoint inhibitors
2. `openfda_label_searcher(name="pembrolizumab")`
3. `openfda_label_getter(set_id="...", sections=["pediatric_use"])`
4. `openfda_adverse_searcher(drug="pembrolizumab")` - Filter for pediatric if possible
5. Repeat for other checkpoint inhibitors

## Complex Queries

### Multi-Drug Regimen Safety

```
Analyze potential safety concerns for the FOLFOX chemotherapy regimen
(5-FU, leucovorin, oxaliplatin) based on FDA adverse event data.
```

**Expected BioMCP Usage:**

1. `think` - Plan multi-drug analysis
2. `openfda_adverse_searcher(drug="fluorouracil")`
3. `openfda_adverse_searcher(drug="leucovorin")`
4. `openfda_adverse_searcher(drug="oxaliplatin")`
5. Identify overlapping toxicities
6. `openfda_label_searcher(name="oxaliplatin")` - Check for combination warnings

### Biomarker-Driven Treatment Selection

```
For a patient with BRAF V600E mutant melanoma with brain metastases,
what FDA-approved treatments are available and what are their CNS-specific
efficacy and safety considerations?
```

**Expected BioMCP Usage:**

1. `think` - Structure biomarker-driven search
2. `article_searcher(genes=["BRAF"], variants=["V600E"], diseases=["melanoma"])`
3. `openfda_label_searcher(indication="melanoma")`
4. For BRAF inhibitors: `openfda_label_getter(set_id="...", sections=["clinical_studies", "warnings_and_precautions"])`
5. `openfda_adverse_searcher(drug="dabrafenib", reaction="seizure")`
6. `openfda_adverse_searcher(drug="vemurafenib", reaction="brain")`

### Treatment Failure Analysis

```
A patient's lung adenocarcinoma progressed on osimertinib. Based on FDA data,
what are the documented resistance mechanisms and alternative approved treatments?
```

**Expected BioMCP Usage:**

1. `think` - Analyze resistance and alternatives
2. `openfda_label_getter(set_id="...", sections=["clinical_studies"])` for osimertinib
3. `article_searcher(genes=["EGFR"], keywords=["resistance", "osimertinib"])`
4. `openfda_label_searcher(indication="non-small cell lung cancer")`
5. `trial_searcher(conditions=["NSCLC"], keywords=["osimertinib resistant"])`

## Safety Monitoring

### Post-Market Surveillance

```
Have there been any new safety signals for CDK4/6 inhibitors
(palbociclib, ribociclib, abemaciclib) in the past year?
```

**Expected BioMCP Usage:**

1. `think` - Plan safety signal detection
2. `openfda_adverse_searcher(drug="palbociclib", limit=100)`
3. `openfda_adverse_searcher(drug="ribociclib", limit=100)`
4. `openfda_adverse_searcher(drug="abemaciclib", limit=100)`
5. Analyze for unusual patterns or frequencies

### Rare Adverse Event Investigation

```
Investigate reports of pneumonitis associated with immune checkpoint inhibitors.
Which drugs have the highest frequency and what are the typical outcomes?
```

**Expected BioMCP Usage:**

1. `think` - Structure pneumonitis investigation
2. `openfda_adverse_searcher(drug="pembrolizumab", reaction="pneumonitis")`
3. `openfda_adverse_searcher(drug="nivolumab", reaction="pneumonitis")`
4. `openfda_adverse_searcher(drug="atezolizumab", reaction="pneumonitis")`
5. Compare frequencies and outcomes
6. `openfda_adverse_getter(report_id="...")` for severe cases

## Quality Assurance

### Diagnostic Test Validation

```
What quality issues have been reported for liquid biopsy ctDNA tests
that could affect treatment decisions?
```

**Expected BioMCP Usage:**

1. `think` - Identify quality factors
2. `openfda_device_searcher(device="liquid biopsy", genomics_only=True)`
3. `openfda_device_searcher(device="ctDNA", genomics_only=True)`
4. `openfda_device_searcher(device="circulating tumor", genomics_only=True)`
5. Analyze failure modes

## Tips for Effective Prompts

1. **Be specific about the data needed**: Specify if you want adverse events, labels, or device data
2. **Include relevant filters**: Mention if focusing on serious events, specific populations, or genomic devices
3. **Request appropriate analysis**: Ask for comparisons, trends, or specific data points
4. **Consider multiple data sources**: Combine OpenFDA with literature and trial data for comprehensive answers
5. **Include time frames when relevant**: Though OpenFDA doesn't support date filtering in queries, you can ask for analysis of recent reports

## Integration Examples

### Combining with Literature Search

```
Find FDA adverse events for venetoclax in CLL, then search for published
case reports that provide more clinical context for the most serious events.
```

### Combining with Clinical Trials

```
What adverse events are reported for FDA-approved CAR-T therapies, and how
do these compare to adverse events being monitored in current clinical trials?
```

### Combining with Variant Data

```
For patients with RET fusion-positive cancers, what FDA-approved targeted
therapies are available and what are their mutation-specific response rates?
```

## Using Your OpenFDA API Key

The OpenFDA API has rate limits: 40 requests/minute without a key, or 240 requests/minute with a key. You can get a free API key at https://open.fda.gov/apis/authentication/

### Method 1: Include API Key in Your Prompt

You can provide your API key directly in your conversation with the AI:

```
My OpenFDA API key is: YOUR_API_KEY_HERE

Search for all serious adverse events reported for pembrolizumab in the last year.
Include both death and hospitalization events.
```

```
Using my OpenFDA API key YOUR_API_KEY_HERE, compare the safety profiles of
all FDA-approved BRAF inhibitors for melanoma treatment.
```

```
I have an OpenFDA API key: YOUR_API_KEY_HERE

Analyze device malfunction reports for all NGS-based cancer diagnostic tests.
Focus on false positive and false negative results that could impact treatment decisions.
```

### Method 2: Set Environment Variable (Recommended for Regular Use)

If you're using BioMCP regularly, set the environment variable once:

```bash
export OPENFDA_API_KEY="YOUR_API_KEY_HERE"
```

Then your prompts don't need to include the key:

```
Retrieve the complete FDA label for trastuzumab deruxtecan, including all
warnings and contraindications for HER2-low breast cancer patients.
```

### Method 3: CLI Usage with API Key

When using the BioMCP CLI directly:

```bash
# Pass API key as parameter
biomcp openfda adverse search --drug pembrolizumab --serious --api-key YOUR_API_KEY_HERE

# Or set environment variable first
export OPENFDA_API_KEY="YOUR_API_KEY_HERE"
biomcp openfda device search --manufacturer Illumina --genomics-only
```

### Example Prompts with API Key

#### Comprehensive Drug Safety Analysis

```
My OpenFDA API key is: YOUR_API_KEY_HERE

Perform a comprehensive safety analysis of sotorasib (Lumakras) including:
1. All serious adverse events from post-market surveillance
2. Complete FDA label with all sections
3. Any device issues if it's a companion diagnostic drug
4. Compare its safety profile to other KRAS G12C inhibitors if available

This is for a clinical review, so I need detailed data from all available FDA sources.
```

#### Large-Scale Adverse Event Analysis

```
Using my OpenFDA API key YOUR_API_KEY_HERE, analyze adverse events for all
FDA-approved checkpoint inhibitors (pembrolizumab, nivolumab, atezolizumab,
durvalumab, avelumab, cemiplimab).

For each drug:
- Get the top 20 most frequent adverse events
- Identify immune-related adverse events
- Check for any black box warnings in their labels
- Note any fatal events

This requires many API calls, so please use my API key for higher rate limits.
```

#### Multi-Device Comparison

```
I have an OpenFDA API key: YOUR_API_KEY_HERE

Compare all FDA adverse event reports for NGS-based companion diagnostic devices
from major manufacturers (Foundation Medicine, Guardant Health, Tempus, etc.).
Focus on:
- Test failure rates
- Sample quality issues
- False positive/negative reports
- Software-related problems

This analysis requires querying multiple device records, so the API key will help
avoid rate limiting.
```

#### Batch Label Retrieval

```
My OpenFDA API key is YOUR_API_KEY_HERE.

Retrieve the complete FDA labels for all CDK4/6 inhibitors (palbociclib,
ribociclib, abemaciclib) and extract:
- Approved indications
- Dose modifications for adverse events
- Drug-drug interactions
- Special population considerations

Then create a comparison table of their safety profiles and dosing guidelines.
```

### When to Provide an API Key

You should provide your API key when:

1. **Performing large-scale analyses** requiring many API calls
2. **Conducting comprehensive safety reviews** across multiple drugs/devices
3. **Running batch operations** like comparing multiple products
4. **Doing rapid iterative searches** that might hit rate limits
5. **Performing systematic reviews** requiring extensive data retrieval

### API Key Security Notes

- Never share your actual API key in public forums or repositories
- The AI will use your key only for the current session
- Keys passed as parameters override environment variables
- The FDA API key is free and can be regenerated if compromised

## Important Notes

- Always expect the AI to use the `think` tool first for complex queries
- The AI should include appropriate disclaimers about adverse events not proving causation
- Results are limited by FDA's data availability and reporting patterns
- The AI should suggest when additional data sources might provide complementary information
- With an API key, you can make 240 requests/minute vs 40 without

## Known Limitations

### Drug Shortage Data

**Important:** The FDA does not currently provide a machine-readable API for drug shortage data. The shortage search tools will return an informative message directing users to the FDA's web-based shortage database. This is a limitation of FDA's current data infrastructure, not a bug in BioMCP.

Alternative resources for drug shortage information:

- FDA Drug Shortages Database: https://www.accessdata.fda.gov/scripts/drugshortages/
- ASHP Drug Shortages: https://www.ashp.org/drug-shortages/current-shortages

### Other Limitations

- Device adverse event reports use abbreviated device names (e.g., "F1CDX" instead of "FoundationOne CDx")
- Adverse event reports represent voluntary submissions and may not reflect true incidence rates
- Recall information may have a delay of 24-48 hours from initial FDA announcement
