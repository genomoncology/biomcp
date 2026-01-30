# Use Case: Precision Oncology Report

Generate a comprehensive report for a patient with a targetable mutation.

## Scenario

A patient with melanoma has BRAF V600E mutation detected. Find actionable information including approved therapies, clinical trials, and relevant literature.

## Workflow

### Step 1: Get Variant Annotations

```bash
biomcp variant search --gene BRAF --hgvsp "p.Val600Glu" --size 5
```

This returns:
- Clinical significance from ClinVar
- Population frequencies from gnomAD
- Cancer annotations from COSMIC
- Prediction scores (CADD, PolyPhen, SIFT)

### Step 2: Find Clinical Trials

```bash
biomcp trial search --condition melanoma --term "BRAF V600" --status open --phase phase3
```

Filter for recruiting phase 3 trials targeting BRAF-mutant melanoma.

### Step 3: Search Literature

```bash
biomcp article search --gene BRAF --disease melanoma --keyword "V600E treatment"
```

Find recent publications on BRAF V600E therapy including resistance mechanisms.

### Step 4: Check Approved Therapies

```bash
biomcp openfda label search --indication melanoma
```

Find FDA-approved drugs with melanoma indications (vemurafenib, dabrafenib, etc.).

## Expected Output

- Variant pathogenicity: Pathogenic (ClinVar)
- Approved drugs: Vemurafenib (ZELBORAF), Dabrafenib + Trametinib
- Active trials: Phase 3 combination studies
- Key literature: Resistance mechanisms, combination strategies

## Tips

- Use `--json` flag to get structured output for programmatic processing
- Combine with `biomcp openfda adverse` to check safety profiles
- Search for "resistance" keyword to find resistance mechanism literature
