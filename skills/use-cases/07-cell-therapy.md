# Use Case: Cell Therapy Trial Landscape

Explore the clinical trial landscape for CAR-T and other cell therapies.

## Scenario

Survey active CAR-T cell therapy trials across indications.

## Workflow

### Step 1: Search CAR-T Trials

```bash
biomcp trial search --term "CAR-T" --status open --phase phase3
```

Find late-stage CAR-T trials.

### Step 2: Filter by Indication

```bash
biomcp trial search --term "CAR-T" --condition "lymphoma" --status open
```

Focus on specific cancer types.

### Step 3: Search by Intervention Type

```bash
biomcp trial search --condition "leukemia" --intervention-type biological --term "cell therapy"
```

Broader search including other cell therapies.

### Step 4: Get Trial Details

```bash
biomcp trial get NCT05727904
```

Review specific trial eligibility and design.

### Step 5: Search Literature

```bash
biomcp article search --keyword "CAR-T therapy" --disease "lymphoma" --page 1
```

Find recent CAR-T publications.

## Cell Therapy Trial Types

| Therapy | Search Term |
|---------|-------------|
| CAR-T | `--term "CAR-T"` |
| TIL | `--term "tumor infiltrating lymphocytes"` |
| TCR-T | `--term "TCR T cell"` |
| NK cell | `--term "NK cell therapy"` |

## Key Indications

- B-cell lymphomas (CD19 CAR-T)
- Multiple myeloma (BCMA CAR-T)
- Acute lymphoblastic leukemia
- Solid tumors (emerging)

## Expected Output

CAR-T trials discovered:
- NCT06904729: CAR-T for lupus nephritis (Phase 3)
- NCT06237336: Relapsed hematological malignancies
- NCT03937544: CD19 CAR-T for B-ALL

## Trial Considerations

| Factor | Notes |
|--------|-------|
| Manufacturing | 2-4 week turnaround for autologous |
| Eligibility | Often requires adequate organ function |
| Monitoring | CRS/ICANS risk requires specialized centers |
| Follow-up | Long-term safety monitoring required |

## Tips

- CAR-T trials often have complex eligibility criteria
- Check for prior therapy requirements
- Note manufacturing timeline in study design
- Review safety monitoring requirements
