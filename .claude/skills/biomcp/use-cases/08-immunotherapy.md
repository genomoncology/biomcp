# Use Case: Immunotherapy Biomarker Trials

Find immunotherapy trials based on biomarker status.

## Scenario

Identify clinical trials for melanoma patients based on PD-L1 expression or other immunotherapy biomarkers.

## Workflow

### Step 1: Search by Biomarker Term

```bash
biomcp trial search --condition "melanoma" --term "PD-L1" --status open
```

Find trials mentioning PD-L1.

### Step 2: Filter for Biologics

```bash
biomcp trial search --condition "melanoma" --term "PD-L1" --status open --intervention-type biological
```

Focus on immunotherapy agents.

### Step 3: Search Checkpoint Inhibitor Trials

```bash
biomcp trial search --condition "melanoma" --term "pembrolizumab" --status open --phase phase3
```

Find trials with specific checkpoint inhibitors.

### Step 4: Search Combination Trials

```bash
biomcp trial search --condition "melanoma" --term "nivolumab ipilimumab" --status open
```

Find combination immunotherapy trials.

### Step 5: Check Drug Safety

```bash
biomcp openfda adverse search --drug nivolumab --serious --limit 10
```

Review safety profile of immunotherapy agents.

## Key Immunotherapy Biomarkers

| Biomarker | Relevance                      |
| --------- | ------------------------------ |
| PD-L1     | Checkpoint inhibitor response  |
| TMB       | Tumor mutational burden        |
| MSI-H     | Microsatellite instability     |
| TILs      | Tumor infiltrating lymphocytes |

## Immunotherapy Drug Classes

| Class             | Examples                 | Target |
| ----------------- | ------------------------ | ------ |
| PD-1 inhibitors   | Pembrolizumab, Nivolumab | PD-1   |
| PD-L1 inhibitors  | Atezolizumab, Durvalumab | PD-L1  |
| CTLA-4 inhibitors | Ipilimumab               | CTLA-4 |
| LAG-3 inhibitors  | Relatlimab               | LAG-3  |

## Expected Output

Immunotherapy trials discovered:

- NCT06961006: V940 vaccine + pembrolizumab (Phase 2)
- NCT05111574: Nivolumab + cabozantinib adjuvant
- NCT05144698: RAPA-201 cell therapy + pembrolizumab

## Immune-Related Adverse Events

Monitor for:

- Pneumonitis
- Colitis
- Hepatitis
- Thyroid dysfunction
- Skin reactions

```bash
biomcp openfda adverse search --drug pembrolizumab --reaction "pneumonitis"
```

## Tips

- Biomarker cutoffs vary by trial (e.g., PD-L1 ≥1% vs ≥50%)
- Check eligibility for prior immunotherapy exposure
- Review immune-related AE requirements
- Consider combination vs monotherapy trials
