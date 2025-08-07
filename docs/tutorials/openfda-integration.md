# OpenFDA Integration Guide

## Overview

BioMCP now integrates with the FDA's openFDA API to provide access to critical drug safety and regulatory information. This integration adds three major data sources to BioMCP's capabilities:

1. **Drug Adverse Events (FAERS)** - FDA Adverse Event Reporting System data
2. **Drug Labels (SPL)** - Official FDA drug product labeling
3. **Device Events (MAUDE)** - Medical device adverse event reports

This guide covers how to use these new tools effectively for precision oncology research.

## Quick Start

### Installation & Setup

The OpenFDA integration is included in the standard BioMCP installation:

```bash
# Install BioMCP
pip install biomcp-python

# Optional: Set API key for higher rate limits
export OPENFDA_API_KEY="your-api-key-here"
```

> **Note**: An API key is optional but recommended. Without one, you're limited to 40 requests/minute. With a key, you get 240 requests/minute. [Get a free API key here](https://open.fda.gov/apis/authentication/).

### Basic Usage Examples

#### Search for drug adverse events

```bash
# Find adverse events for a specific drug
biomcp openfda adverse search --drug imatinib

# Search for specific reactions
biomcp openfda adverse search --reaction nausea --serious

# Get detailed report
biomcp openfda adverse get REPORT123456
```

#### Search drug labels

```bash
# Find drugs for specific indications
biomcp openfda label search --indication melanoma

# Search for drugs with boxed warnings
biomcp openfda label search --boxed-warning

# Get complete label
biomcp openfda label get SET_ID_HERE
```

#### Search device events

```bash
# Search for genomic test device issues
biomcp openfda device search --device "FoundationOne"

# Search by manufacturer
biomcp openfda device search --manufacturer Illumina

# Get detailed device event
biomcp openfda device get MDR123456
```

## MCP Tool Usage

### For AI Agents

The OpenFDA tools are available as MCP tools for AI agents. Each tool includes built-in reminders to use the `think` tool first for complex queries.

#### Available Tools

- `openfda_adverse_searcher` - Search drug adverse events
- `openfda_adverse_getter` - Get specific adverse event report
- `openfda_label_searcher` - Search drug labels
- `openfda_label_getter` - Get complete drug label
- `openfda_device_searcher` - Search device adverse events
- `openfda_device_getter` - Get specific device event report

#### Example Tool Usage

```python
# Search for adverse events
result = await openfda_adverse_searcher(
    drug="pembrolizumab",
    serious=True,
    limit=25
)

# Get drug label
label = await openfda_label_getter(
    set_id="abc-123-def",
    sections=["indications_and_usage", "warnings_and_precautions"]
)

# Search genomic devices
devices = await openfda_device_searcher(
    device="sequencer",
    genomics_only=True,  # Filter to genomic/diagnostic devices
    problem="false positive"
)
```

## Data Sources Explained

### Drug Adverse Events (FAERS)

The FDA Adverse Event Reporting System contains reports of adverse events and medication errors submitted to FDA. Key features:

- **Voluntary reporting**: Reports come from healthcare professionals, patients, and manufacturers
- **No causation proof**: Reports don't establish that a drug caused the event
- **Rich detail**: Includes patient demographics, drug information, reactions, and outcomes
- **Real-world data**: Captures post-market safety signals

**Best for**: Understanding potential side effects, safety signals, drug interactions

### Drug Labels (SPL)

Structured Product Labeling contains the official FDA-approved prescribing information. Includes:

- **Indications and usage**: FDA-approved uses
- **Dosage and administration**: How to prescribe
- **Contraindications**: When not to use
- **Warnings and precautions**: Safety information
- **Drug interactions**: Known interactions
- **Clinical studies**: Trial data supporting approval

**Best for**: Official prescribing guidelines, approved indications, contraindications

### Device Events (MAUDE)

Manufacturer and User Facility Device Experience database contains medical device adverse events. For BioMCP, we focus on genomic/diagnostic devices:

- **Genomic test devices**: Issues with sequencing platforms, diagnostic panels
- **In vitro diagnostics**: Problems with biomarker tests
- **Device malfunctions**: Technical failures affecting test results
- **Patient impact**: How device issues affected patient care

**Best for**: Understanding reliability of genomic tests, device-related diagnostic issues

## Advanced Features

### Genomic Device Filtering

By default, device searches filter to genomic/diagnostic devices relevant to precision oncology:

```bash
# Search only genomic devices (default)
biomcp openfda device search --device test

# Search ALL medical devices
biomcp openfda device search --device test --all-devices
```

The genomic filter includes FDA product codes for:

- Next Generation Sequencing panels
- Gene mutation detection systems
- Tumor profiling tests
- Hereditary variant detection systems

### Pagination Support

All search tools support pagination for large result sets:

```bash
# Get second page of results
biomcp openfda adverse search --drug aspirin --page 2 --limit 50
```

### Section-Specific Label Retrieval

When retrieving drug labels, you can specify which sections to include:

```bash
# Get only specific sections
biomcp openfda label get SET_ID --sections "indications_and_usage,adverse_reactions"
```

## Integration with Other BioMCP Tools

### Complementary Data Sources

OpenFDA data complements existing BioMCP tools:

| Tool                       | Data Source        | Best For                          |
| -------------------------- | ------------------ | --------------------------------- |
| `drug_getter`              | MyChem.info        | Chemical properties, mechanisms   |
| `openfda_label_searcher`   | FDA Labels         | Official indications, prescribing |
| `openfda_adverse_searcher` | FAERS              | Safety signals, side effects      |
| `trial_searcher`           | ClinicalTrials.gov | Active trials, eligibility        |

### Workflow Examples

#### Complete Drug Profile

```python
# 1. Get drug chemical info
drug_info = await drug_getter("imatinib")

# 2. Get FDA label
label = await openfda_label_searcher(name="imatinib")

# 3. Check adverse events
safety = await openfda_adverse_searcher(drug="imatinib", serious=True)

# 4. Find current trials
trials = await trial_searcher(interventions=["imatinib"])
```

#### Device Reliability Check

```python
# 1. Search for device issues
events = await openfda_device_searcher(
    device="FoundationOne CDx",
    problem="false"
)

# 2. Get specific event details
if events:
    details = await openfda_device_getter("MDR_KEY_HERE")
```

## Important Considerations

### Data Limitations

1. **Adverse Events**:

   - Reports don't prove causation
   - Reporting is voluntary, so not all events are captured
   - Duplicate reports may exist
   - Include appropriate disclaimers when presenting data

2. **Drug Labels**:

   - May not reflect the most recent changes
   - Off-label uses not included
   - Generic drugs may have different inactive ingredients

3. **Device Events**:
   - Not all device problems are reported
   - User error vs device malfunction can be unclear
   - Reports may lack complete information

### Rate Limits

- **Without API key**: 40 requests/minute per IP
- **With API key**: 240 requests/minute per key
- **Burst limit**: 4 requests/second

### Best Practices

1. **Always use disclaimers**: Include FDA's disclaimer about adverse events not proving causation
2. **Check multiple sources**: Combine OpenFDA data with other BioMCP tools
3. **Filter appropriately**: Use genomic device filtering for relevant results
4. **Handle no results gracefully**: Many specific queries may return no results
5. **Respect rate limits**: Use API key for production use

## Troubleshooting

### Common Issues

**No results found**

- Try broader search terms
- Check spelling of drug/device names
- Remove filters to expand search

**Rate limit errors**

- Add API key to environment
- Reduce request frequency
- Batch queries when possible

**Timeout errors**

- OpenFDA API may be slow/down
- Retry after a brief wait
- Consider caching frequent queries

### Getting Help

- OpenFDA documentation: https://open.fda.gov/apis/
- OpenFDA status: https://api.fda.gov/status
- BioMCP issues: https://github.com/genomoncology/biomcp/issues

## API Reference

### Environment Variables

- `OPENFDA_API_KEY`: Your openFDA API key (optional but recommended)

### CLI Commands

```bash
# Adverse Events
biomcp openfda adverse search [OPTIONS]
  --drug TEXT           Drug name to search
  --reaction TEXT       Reaction to search
  --serious/--all       Filter serious events
  --limit INT           Results per page (max 100)
  --page INT            Page number

biomcp openfda adverse get REPORT_ID

# Drug Labels
biomcp openfda label search [OPTIONS]
  --name TEXT           Drug name
  --indication TEXT     Indication to search
  --boxed-warning       Has boxed warning
  --section TEXT        Label section
  --limit INT           Results per page
  --page INT            Page number

biomcp openfda label get SET_ID [OPTIONS]
  --sections TEXT       Comma-separated sections

# Device Events
biomcp openfda device search [OPTIONS]
  --device TEXT         Device name
  --manufacturer TEXT   Manufacturer name
  --problem TEXT        Problem description
  --product-code TEXT   FDA product code
  --genomics-only/--all-devices
  --limit INT           Results per page
  --page INT            Page number

biomcp openfda device get MDR_KEY
```

## Example Outputs

### Adverse Event Search

```markdown
## FDA Adverse Event Reports

**Drug**: imatinib | **Serious Events**: Yes
**Total Reports Found**: 1,234 reports

### Top Reported Reactions:

- **NAUSEA**: 234 reports (19.0%)
- **FATIGUE**: 189 reports (15.3%)
- **RASH**: 156 reports (12.6%)

### Sample Reports (showing 3 of 1,234):

...
```

### Drug Label Search

```markdown
## FDA Drug Labels

**Drug**: pembrolizumab
**Total Labels Found**: 5 labels

### Results (showing 5 of 5):

#### 1. KEYTRUDA

**Also known as**: pembrolizumab
**FDA Application**: BLA125514
**Manufacturer**: Merck Sharp & Dohme
**Route**: INTRAVENOUS

⚠️ **BOXED WARNING**: Immune-mediated adverse reactions...

**Indications**: KEYTRUDA is indicated for the treatment of...
```

### Device Event Search

```markdown
## FDA Device Adverse Event Reports

**Device**: FoundationOne | **Type**: Genomic/Diagnostic Devices
**Total Reports Found**: 12 reports

### Top Reported Problems:

- **False negative result**: 5 reports (41.7%)
- **Software malfunction**: 3 reports (25.0%)

### Sample Reports (showing 3 of 12):

...
```
