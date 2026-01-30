# Use Case: Hereditary Cancer Syndromes

Research hereditary cancer syndromes including variant interpretation and surveillance trials.

## Scenario

Investigate Lynch syndrome, a hereditary cancer predisposition syndrome, including causative variants and clinical trials.

## Workflow

### Step 1: Search Clinical Trials

```bash
biomcp trial search --condition "Lynch syndrome" --status open
```

Find trials for Lynch syndrome patients.

### Step 2: Search Causative Gene Variants

```bash
biomcp variant search --gene MLH1 --significance pathogenic --size 10
```

Find pathogenic variants in mismatch repair genes.

### Step 3: Search Other MMR Genes

```bash
biomcp variant search --gene MSH2 --significance pathogenic --size 5
biomcp variant search --gene MSH6 --significance pathogenic --size 5
biomcp variant search --gene PMS2 --significance pathogenic --size 5
```

### Step 4: Get Disease Information

```bash
biomcp disease search "Lynch syndrome"
```

Get syndrome details and synonyms.

### Step 5: Search Literature

```bash
biomcp article search --gene MLH1 --disease "colorectal cancer" --keyword "Lynch" --page 1
```

Find Lynch syndrome research publications.

## Lynch Syndrome Genes

| Gene  | Associated Cancers                     |
| ----- | -------------------------------------- |
| MLH1  | Colorectal, endometrial, ovarian       |
| MSH2  | Colorectal, endometrial, urinary tract |
| MSH6  | Colorectal, endometrial                |
| PMS2  | Colorectal (lower penetrance)          |
| EPCAM | Colorectal (MSH2 silencing)            |

## Hereditary Cancer Syndromes

| Syndrome    | Genes                  | Key Cancers             |
| ----------- | ---------------------- | ----------------------- |
| Lynch       | MLH1, MSH2, MSH6, PMS2 | Colorectal, endometrial |
| HBOC        | BRCA1, BRCA2           | Breast, ovarian         |
| FAP         | APC                    | Colorectal              |
| Li-Fraumeni | TP53                   | Multiple                |
| Cowden      | PTEN                   | Breast, thyroid         |

## Expected Output

Lynch syndrome findings:

- Pathogenic MLH1 variants: Truncating mutations, splice site variants
- Active trials: Surveillance studies, prevention trials, immunotherapy
- Cancer risks: 50-80% lifetime colorectal cancer risk

## Trial Types for Hereditary Syndromes

| Type         | Purpose                         |
| ------------ | ------------------------------- |
| Surveillance | Optimizing screening protocols  |
| Prevention   | Chemoprevention, risk reduction |
| Diagnostic   | Biomarker development           |
| Treatment    | Cancer-specific therapies       |

## Example Trials

Lynch syndrome trials discovered:

- NCT07360834: Liquid biopsy surveillance study
- NCT06582914: LINEAGE epidemiology consortium
- NCT07163403: Dendritic cell vaccine prevention

## Tips

- Search all genes in a syndrome panel
- Include both syndrome name and gene names in searches
- Check for prevention/risk reduction trials
- Review surveillance protocol trials
- Consider family testing implications
