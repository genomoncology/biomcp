# Use Case: Clinical Trial Matching

Find eligible clinical trials for a patient with specific molecular profile.

## Scenario

A patient with non-small cell lung cancer (NSCLC) has an EGFR exon 19 deletion. Find recruiting trials that accept this mutation.

## Workflow

### Step 1: Search by Condition and Mutation

```bash
biomcp trial search --condition "non-small cell lung cancer" --term "EGFR mutation" --status open
```

### Step 2: Filter by Phase

```bash
biomcp trial search --condition "non-small cell lung cancer" --term "EGFR" --status open --phase phase3
```

### Step 3: Search with Biomarker Requirements

```bash
biomcp trial search --condition "lung cancer" --required-mutation "EGFR" --status open
```

### Step 4: Location-Based Search

```bash
biomcp trial search --condition "lung cancer" --term "EGFR" --status open \
  --lat 42.3601 --lon -71.0589 --distance 100
```

Find trials within 100 miles of Boston.

### Step 5: Get Trial Details

```bash
biomcp trial get NCT06422546
```

Get full eligibility criteria, contacts, and study design.

## Key Filters

| Filter | Purpose |
|--------|---------|
| `--status open` | Only recruiting trials |
| `--phase phase3` | Late-stage trials |
| `--intervention-type biological` | Biologics/immunotherapy |
| `--age-group adult` | Adult patients only |
| `--lat/--lon/--distance` | Geographic proximity |

## Expected Output

- Trial NCT numbers with brief summaries
- Eligibility criteria including mutation requirements
- Study phase and enrollment status
- Site locations and contact information

## Tips

- Use `--required-mutation` for mutation-specific trials
- Combine `--condition` with `--term` for precise matching
- Check `--intervention-type` to filter by drug class
