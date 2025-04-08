# Getting Started with BioMCP

Welcome to BioMCP! This guide will walk you through your first steps using the command-line interface (CLI) to access biomedical data quickly.

## Prerequisite

Ensure you have successfully installed BioMCP via the `biomcp-python` package. If not, please follow the [Installation Guide](installation.md).

## Basic Commands

BioMCP provides commands for interacting with different data types: `variant`, `trial`, and `article`. Let's try one example for each.

### 1. Finding a Genetic Variant

You can search for variants using gene names, protein changes, rsIDs, and more.

**Command:** Find the common BRAF V600E mutation.

```bash
biomcp variant search --gene BRAF --protein p.V600E
```

Example Output (Truncated Markdown):

```markdown
# Record 1

## \_Id: Chr7:G.140453136a>T

### CADD

Phred: 32

### Clinvar

#### Rcv

Clinical Significance: Pathogenic

### Dbnsfp

Genename: BRAF
Hgvsc: c.1799T>A
Hgvsp: p.V600E

### Dbsnp

Rsid: rs113488022

### Exac

Af: 1.647e-05

### Gnomad Exome

#### Af

Af: 3.97994e-06

... (more annotations) ...
```

### 2. Finding Clinical Trials

Search for clinical trials based on conditions, interventions, recruitment status, etc.

**Command:** Find currently recruiting trials for Melanoma.

```bash
biomcp trial search --condition "Melanoma" --status OPEN
```

Example Output (Truncated Markdown):

```markdown
# Record 1

Nct Number: NCT06212009
Study Title: A Study of DF1001 in Patients With Advanced Solid Tumors
Study Url: https://clinicaltrials.gov/study/NCT06212009
Study Status: RECRUITING
Brief Summary:
This is a Phase 1/2 first-in-human, multi-center, open-label study
of DF1001 in patients with advanced solid tumors. The study consists
of dose escalation (Phase 1) and dose expansion (Phase 2). Phase 1
will determine the maximum tolerated dose (MTD) and recommended Phase 2
dose (RP2D) of DF1001 as monotherapy and in combination with nivolumab.
Phase 2 will evaluate the safety and efficacy of DF1001 at the RP2D as
monotherapy and in combination with nivolumab in selected indications.
Study Results: No Results Available
Conditions:

- Advanced Solid Tumor
- Head and Neck Squamous Cell Carcinoma
- Non-small Cell Lung Cancer
- Melanoma
- Urothelial Carcinoma
- Cervical Cancer
- Endometrial Cancer
  Interventions:
- Drug: DF1001
- Biological: nivolumab
  Phases:
- Phase1
- Phase2
  Enrollment: 198
  Study Type: Interventional
  Study Design:
  Allocation: Non-Randomized, Endpoint Classification: Safety/Efficacy
  Study, Intervention Model: Parallel Assignment, Masking: None (Open
  Label), Primary Purpose: Treatment
  Start Date: 2023-12-19
  Completion Date: 2027-12-31

... (more records) ...
```

### 3. Finding Research Articles

Search for biomedical literature using genes, diseases, chemicals, variants, or keywords.

**Command:** Find recent articles about TP53 in Lung Cancer.

```bash
biomcp article search --gene TP53 --disease "Lung Cancer"
```

Example Output (Truncated Markdown):

```markdown
# Record 1

Pmid: 39034737
Title:
Distinct Clinical Outcomes Associated With Non-V600 BRAF Mutations
in Patients With Non-Small-Cell Lung Cancer in Real-World Settings.
Journal: Clin Lung Cancer
Authors:

- Zhao S
- Fang W
- Huang Y
- [...]
  Date: 2024-11-01T00:00:00Z
  Doi: 10.1016/j.cllc.2024.08.007
  Abstract:
  INTRODUCTION: Non-V600 BRAF mutations have distinct kinase activities,
  transforming potential, and therapeutic vulnerabilities. This study
  evaluated the clinical outcomes of non-small-cell lung cancer (NSCLC)
  patients with non-V600 BRAF mutations. PATIENTS AND METHODS: [...] We
  also assessed the impact of concurrent mutations (eg, TP53) on outcomes.
  RESULTS: [...] Concurrent TP53 mutation was associated with numerically
  shorter progression-free survival and overall survival in patients with
  Class II/III mutations treated with systemic therapy. CONCLUSION: NSCLC
  patients harboring non-V600 BRAF mutations exhibit heterogeneous
  clinicopathologic features and treatment outcomes. Concurrent TP53
  mutation might indicate poor prognosis. [...]
  Pubmed Url: https://pubmed.ncbi.nlm.nih.gov/39034737/
  Doi Url: https://doi.org/10.1016/j.cllc.2024.08.007

... (more records) ...
```

## Understanding the Output

By default, BioMCP outputs results in Markdown format, designed for easy reading in the terminal.

For programmatic use or integration with other tools, you can often request JSON output using the `--json` flag:

```bash
biomcp variant search --gene BRAF --protein p.V600E --json
```

## Next Steps

You've now seen the basics of BioMCP! To learn more:

- Explore practical examples in [Common Workflows](workflows.md).
- See all available options for each command:
  - [Trials CLI Reference](cli/trials.md)
  - [Articles CLI Reference](cli/articles.md)
  - [Variants CLI Reference](cli/variants.md)
- Learn about the server component in the [Server Protocol Guide](server_protocol.md) and [MCP Integration Guide](mcp_integration.md).
