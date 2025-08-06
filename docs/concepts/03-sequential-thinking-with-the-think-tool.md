# Sequential Thinking with the Think Tool

## CRITICAL: The Think Tool is MANDATORY

**The 'think' tool must be your FIRST action when using BioMCP. This is not optional.**

For detailed technical documentation on the think tool parameters and usage, see the [MCP Tools Reference - Think Tool](../user-guides/02-mcp-tools-reference.md#3-think).

## Why Sequential Thinking?

Biomedical research is inherently complex, requiring systematic analysis of interconnected data from multiple sources. The think tool enforces a structured approach that:

- **Prevents Information Overload**: Breaks complex queries into manageable steps
- **Ensures Comprehensive Coverage**: Systematic thinking catches details that might be missed
- **Documents Reasoning**: Creates an audit trail of research decisions
- **Improves Accuracy**: Thoughtful planning leads to better search strategies

## Mandatory Usage Requirements

🚨 **REQUIRED USAGE:**

- You MUST call 'think' BEFORE any search or fetch operations
- EVERY biomedical research query requires thinking first
- ALL multi-step analyses must begin with the think tool
- ANY task using BioMCP tools requires prior planning with think

⚠️ **WARNING - Skipping the think tool will result in:**

- Incomplete analysis
- Poor search strategies
- Missing critical connections
- Suboptimal results
- Frustrated users

## How to Use the Think Tool

The think tool accepts these parameters:

```python
think(
    thought="Your reasoning about the current step",
    thoughtNumber=1,  # Sequential number starting from 1
    totalThoughts=5,  # Optional: estimated total thoughts needed
    nextThoughtNeeded=True  # Set to False only when analysis is complete
)
```

## Sequential Thinking Patterns

### Pattern 1: Initial Query Decomposition

Always start by breaking down the user's query:

```python
# User asks: "What are the treatment options for BRAF V600E melanoma?"

think(
    thought="Breaking down query: Need to find 1) BRAF V600E mutation significance in melanoma, 2) approved treatments for BRAF-mutant melanoma, 3) clinical trials for new therapies, 4) resistance mechanisms and combination strategies",
    thoughtNumber=1,
    nextThoughtNeeded=True
)
```

### Pattern 2: Search Strategy Planning

Plan your data collection approach:

```python
think(
    thought="Search strategy: First use gene_getter for BRAF context, then article_searcher for BRAF V600E melanoma treatments focusing on FDA-approved drugs, followed by trial_searcher for ongoing studies with BRAF inhibitors",
    thoughtNumber=2,
    nextThoughtNeeded=True
)
```

### Pattern 3: Progressive Refinement

Document findings and adjust strategy:

```python
think(
    thought="Found 3 FDA-approved BRAF inhibitors (vemurafenib, dabrafenib, encorafenib). Need to search for combination therapies with MEK inhibitors based on resistance patterns identified in literature",
    thoughtNumber=3,
    nextThoughtNeeded=True
)
```

### Pattern 4: Synthesis Planning

Before creating final output:

```python
think(
    thought="Ready to synthesize: Will organize findings into 1) First-line treatments (BRAF+MEK combos), 2) Second-line options (immunotherapy), 3) Emerging therapies from trials, 4) Resistance mechanisms to consider",
    thoughtNumber=4,
    nextThoughtNeeded=False  # Analysis complete
)
```

## Common Think Tool Workflows

### Literature Review Workflow

```python
# Step 1: Problem definition
think(thought="User wants comprehensive review of CDK4/6 inhibitors in breast cancer...", thoughtNumber=1)

# Step 2: Search parameters
think(thought="Will search for palbociclib, ribociclib, abemaciclib in HR+/HER2- breast cancer...", thoughtNumber=2)

# Step 3: Quality filtering
think(thought="Found 47 articles, filtering for Phase III trials and meta-analyses...", thoughtNumber=3)

# Step 4: Evidence synthesis
think(thought="Identified consistent PFS benefit across trials, now analyzing OS data...", thoughtNumber=4)
```

### Clinical Trial Analysis Workflow

```python
# Step 1: Criteria identification
think(thought="Patient has EGFR L858R lung cancer, progressed on osimertinib...", thoughtNumber=1)

# Step 2: Trial search strategy
think(thought="Searching for trials accepting EGFR-mutant NSCLC after TKI resistance...", thoughtNumber=2)

# Step 3: Eligibility assessment
think(thought="Found 12 trials, checking for brain metastases eligibility...", thoughtNumber=3)

# Step 4: Prioritization
think(thought="Ranking trials by proximity, novel mechanisms, and enrollment status...", thoughtNumber=4)
```

### Variant Interpretation Workflow

```python
# Step 1: Variant identification
think(thought="Analyzing TP53 R248Q mutation found in patient's tumor...", thoughtNumber=1)

# Step 2: Database queries
think(thought="Will check MyVariant for population frequency, cBioPortal for cancer prevalence...", thoughtNumber=2)

# Step 3: Functional assessment
think(thought="Variant is pathogenic, affects DNA binding domain, common in multiple cancers...", thoughtNumber=3)

# Step 4: Clinical implications
think(thought="Synthesizing prognostic impact and potential therapeutic vulnerabilities...", thoughtNumber=4)
```

## Think Tool Best Practices

### DO:

- Start EVERY BioMCP session with think
- Use sequential numbering (1, 2, 3...)
- Document key findings in each thought
- Adjust strategy based on intermediate results
- Use think to track progress through complex analyses

### DON'T:

- Skip think and jump to searches
- Use think only at the beginning
- Set nextThoughtNeeded=false prematurely
- Use generic thoughts without specific content
- Forget to document decision rationale

## Integration with Other Tools

The think tool should wrap around other tool usage:

```python
# CORRECT PATTERN
think(thought="Planning BRAF melanoma research...", thoughtNumber=1)
gene_info = gene_getter("BRAF")

think(thought="BRAF is a serine/threonine kinase, V600E creates constitutive activation. Searching for targeted therapies...", thoughtNumber=2)
articles = article_searcher(genes=["BRAF"], diseases=["melanoma"], keywords=["vemurafenib", "dabrafenib"])

think(thought="Found key trials showing BRAF+MEK combination superiority. Checking for active trials...", thoughtNumber=3)
trials = trial_searcher(conditions=["melanoma"], interventions=["BRAF inhibitor"])

# INCORRECT PATTERN - NO THINKING
gene_info = gene_getter("BRAF")  # ❌ Started without thinking
articles = article_searcher(...)  # ❌ No strategy planning
```

## Reminder System

BioMCP includes automatic reminders if you forget to use think:

- Search results will include a reminder message
- The reminder appears as a system message
- It prompts you to use think for better results
- This ensures consistent methodology

## Advanced Sequential Thinking

### Branching Logic

Use think to handle conditional paths:

```python
think(
    thought="No direct trials found for this rare mutation. Pivoting to search for basket trials and mutation-agnostic approaches...",
    thoughtNumber=5,
    nextThoughtNeeded=True
)
```

### Error Recovery

Document and adjust when searches fail:

```python
think(
    thought="MyVariant query failed for this structural variant. Will use article search to find functional studies instead...",
    thoughtNumber=6,
    nextThoughtNeeded=True
)
```

### Complex Integration

Coordinate multiple data sources:

```python
think(
    thought="Integrating findings: cBioPortal shows 15% frequency in lung adenocarcinoma, articles describe resistance mechanisms, trials testing combination strategies...",
    thoughtNumber=7,
    nextThoughtNeeded=True
)
```

## Conclusion

The think tool is not just a requirement—it's your research companion that ensures systematic, thorough, and reproducible biomedical research. By following sequential thinking patterns, you'll deliver comprehensive insights that address all aspects of complex biomedical queries.

Remember: **Always think first, then search. Document your reasoning. Only mark thinking complete when your analysis is truly finished.**
