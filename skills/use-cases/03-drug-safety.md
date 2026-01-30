# Use Case: Drug Safety Investigation

Investigate adverse events and safety signals for a specific drug.

## Scenario

Evaluate the safety profile of pembrolizumab (Keytruda), a PD-1 inhibitor used in multiple cancer types.

## Workflow

### Step 1: Search Serious Adverse Events

```bash
biomcp openfda adverse search --drug pembrolizumab --serious --limit 25
```

Returns:
- Total report count
- Top reported reactions
- Sample reports with patient demographics

### Step 2: Search Specific Reaction

```bash
biomcp openfda adverse search --drug pembrolizumab --reaction "pneumonitis"
```

Focus on a known immune-related adverse event.

### Step 3: Get Drug Label

```bash
biomcp openfda label search --name pembrolizumab
```

Review official prescribing information including boxed warnings.

### Step 4: Compare to Similar Drug

```bash
biomcp openfda adverse search --drug nivolumab --serious --limit 25
```

Compare safety profile to another PD-1 inhibitor.

### Step 5: Check Recent Literature

```bash
biomcp article search --chemical pembrolizumab --keyword "adverse events" --page 1
```

Find recent publications on safety data.

## Key Adverse Event Fields

| Field | Description |
|-------|-------------|
| `serious` | Life-threatening, hospitalization, disability |
| `seriousnessdeath` | Fatal outcome |
| `seriousnesshospitalization` | Required hospitalization |
| `receivedate` | Date FDA received report |

## Expected Output

- Report counts: 80,000+ serious reports for pembrolizumab
- Top reactions: Immune-related AEs (pneumonitis, colitis, hepatitis)
- Patient demographics: Age distribution, sex
- Outcome severity: Death, hospitalization rates

## Tips

- Use `--serious` to filter for serious events only
- FAERS data includes all co-medications, not just suspected drugs
- Report dates reflect FDA receipt, not event occurrence
- Combine with literature search for clinical context
