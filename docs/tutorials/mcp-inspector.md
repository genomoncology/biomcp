# Testing BioMCP with MCP Inspector: Step-by-Step Tutorial

This tutorial guides you through using the MCP Inspector to test and debug
BioMCP integration. The MCP Inspector provides a user-friendly interface to
interact with BioMCP directly, without needing to integrate with an AI
assistant first.

## What is the MCP Inspector?

The MCP Inspector is a tool developed by Anthropic that allows you to:

- Test MCP servers like BioMCP directly
- Browse available tools and their parameters
- Send test requests and view responses
- Debug MCP server integrations

## Step 1: Start MCP Inspector

To run biomcp inside the MCP inspector, use the following command:

```bash
npx @modelcontextprotocol/inspector uv run --with biomcp-python biomcp run
```

This will launch the inspector interface (usually at http://127.0.0.1:6274).

## Step 2: Connect

Press the "Connect" button to establish a connection to the MCP server.

## Step 3: List Tools

The Inspector should display the available BioMCP tools:

- `article_searcher`
- `article_details`
- `trial_searcher`
- `trial_protocol`
- `trial_locations`
- `trial_outcomes`
- `trial_references`
- `variant_searcher`
- `variant_details`
- `sequential_thinking`

Click on any tool to see its description and input parameters.

## Step 4: Test BioMCP Tools

Let's test each BioMCP tool with example inputs. For each test:

1. Select the tool from the list
2. Copy and paste the corresponding JSON input
3. Click "Call Tool" to send the request
4. Review the response in the Output section

### Tool 1: article_searcher

**Input:**

```json
{
  "genes": ["EGFR"],
  "diseases": ["NSCLC"],
  "variants": ["BRAF V600E"],
  "keywords": ["MEK Inhibitors"],
  "chemicals": ["Afatinib"]
}
```

**Example Output:**

```markdown
# Record 1

Pmid: 33402199
Pmcid: PMC7786519
Title: MEK inhibitors for the treatment of non-small cell lung cancer
Journal: J Hematol Oncol
Date: 2021-01-05T00:00:00Z
Doi: 10.1186/s13045-020-01025-7
Abstract:
BRAF and KRAS are two key oncogenes in the RAS/RAF/MEK/MAPK signaling
pathway. Concomitant mutations in both KRAS and BRAF genes have been
```

### Tool 2: article_details

**Input:**

```text
21717063
```

**Example Output:**

Same as `article_search` first record but with Full Text after the abstract:

```markdown
Full Text:
Introduction
Lung cancer is the most common cause of cancer-related death worldwide,...
```

### Tool 3: trial_searcher

**Input:**

```json
{
  "conditions": ["Lung Cancer"],
  "interventions": ["Pembrolizumab"],
  "recruiting_status": "OPEN",
  "phase": "PHASE3"
}
```

**Example Output:**

```markdown
# Record 1

Nct Number: NCT06847334
Study Title:
A Study to Compare the Efficacy, Safety, Immunogenicity, and
Pharmacokinetic Profile of HLX17 Vs. Keytruda® in the First-Line
Treatment of Advanced Non-squamous Non-small Cell Lung Cancer
Study Url: https://clinicaltrials.gov/study/NCT06847334
Study Status: NOT_YET_RECRUITING
Brief Summary:
This is a multicentre, randomized, double-blind, parallel-controlled
integrated phase I/III clinical study to evaluate the similarity in
efficacy, safety, PK profile, and immunogenicity of HLX17 vs. Keytruda®(
US- and EU-sourced) in the first-line treatment of advanced non-squamous
...
```

### Tool 4: trial_protocol

### Tool 5: trial_locations

### Tool 6: trial_outcomes

### Tool 7: trial_references

All trial "detail" tools work the same way by specifying the NCT ID.

**Input:**

```text
NCT04280705
```

**Example Output:**

```markdown
Url: https://clinicaltrials.gov/study/NCT04280705

# Protocol Section

## Identification Module

Nct Id: NCT04280705
Brief Title: Adaptive COVID-19 Treatment Trial (ACTT)
Official Title:
A Multicenter, Adaptive, Randomized Blinded Controlled Trial of the
Safety and Efficacy of Investigational Therapeutics for the Treatment of
COVID-19 in Hospitalized Adults
...
```

### Tool 8: variant_searcher

**Input:**

```json
{
  "gene": "BRAF",
  "hgvsp": "p.V600E",
  "size": 5
}
```

### Tool 9: variant_details

**Input:**

```text
rs113488022
```

or

```text
chr7:g.140453136A>T
```

### Tool 10: sequential_thinking

**Input:**

```json
{
  "thought": "First, I need to understand the clinical context of this BRAF V600E mutation",
  "nextThoughtNeeded": true,
  "thoughtNumber": 1,
  "totalThoughts": 3,
  "isRevision": false
}
```

**Example Output:**

```text
Added thought 1 to main sequence. Progress: 1/3 thoughts. Next thought needed.
```

### Testing Location-Based Search

**Tool**: `trial_searcher`
**Input**:

```json
{
  "conditions": ["Breast Cancer"],
  "recruiting_status": "OPEN",
  "lat": 42.3601,
  "long": -71.0589,
  "distance": 100
}
```

### Testing Variant Filtering

**Tool**: `variant_searcher`
**Input**:

```json
{
  "gene": "TP53",
  "significance": "pathogenic",
  "max_frequency": 0.01,
  "cadd": 20,
  "size": 10
}
```
