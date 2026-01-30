# Use Case: Rare Disease Research

Find clinical trials and research for rare diseases with limited treatment options.

## Scenario

Research treatment options and trials for Rett syndrome, a rare neurodevelopmental disorder.

## Workflow

### Step 1: Search Clinical Trials

```bash
biomcp trial search --condition "Rett syndrome" --status open
```

Find all recruiting trials for the condition.

### Step 2: Filter by Intervention Type

```bash
biomcp trial search --condition "Rett syndrome" --intervention-type genetic --status open
```

Focus on gene therapy trials.

### Step 3: Get Disease Information

```bash
biomcp disease search "Rett syndrome"
```

Get disease ontology terms and synonyms.

### Step 4: Search Literature

```bash
biomcp article search --disease "Rett syndrome" --keyword "treatment" --page 1
```

Find recent therapeutic research.

### Step 5: Check Related Genes

```bash
biomcp variant search --gene MECP2 --significance pathogenic --size 5
```

Search variants in the causative gene (MECP2 for Rett syndrome).

## Rare Disease Trial Strategies

| Strategy | Command Example |
|----------|-----------------|
| Gene therapy | `--intervention-type genetic` |
| Natural history | `--type observational` |
| Expanded access | `--type expanded_access` |
| Pediatric | `--age-group child` |

## Expected Output

- Active trials: Gene therapy (AAV-MECP2), trofinetide studies
- Trial phases: Mostly Phase 1/2 for rare diseases
- Endpoints: Safety, biomarkers, functional outcomes
- Literature: Mechanism studies, case reports

## Example Findings

Rett syndrome trials discovered:
- NCT06856759: AAV-MECP2 gene therapy (Phase 1)
- NCT06704816: Trofinetide cognitive assessment
- NCT04900493: Global patient registry

## Tips

- Rare diseases often have few trials - use broad searches
- Check for patient registries (`--type observational`)
- Gene therapy trials are often early phase
- Search by causative gene for variant-specific trials
- Use disease synonyms for comprehensive coverage
