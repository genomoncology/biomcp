# Pattern: Rare Disease Research

Find trials and research for rare diseases with limited treatment options.

## Quick Overview (2 commands)

```bash
# What gene causes it?
biomcp disease get "<disease name>"

# Any recruiting trials?
biomcp trial search --condition "<disease>" --status OPEN
```

## Gene Therapy Trials

```bash
biomcp trial search --condition "<disease>" --intervention-type GENETIC --status OPEN
```

## Full Workflow (run separately)

```bash
# Step 1: Disease info and causative gene
biomcp disease get "<disease>"

# Step 2: Gene therapy trials
biomcp trial search --condition "<disease>" --intervention-type GENETIC

# Step 3: Natural history / registries
biomcp trial search --condition "<disease>" --type OBSERVATIONAL
```

## Trial Strategies for Rare Diseases

| Strategy        | Filter                        |
| --------------- | ----------------------------- |
| Gene therapy    | `--intervention-type genetic` |
| Natural history | `--type observational`        |
| Expanded access | `--type expanded_access`      |
| Pediatric       | `--age-group child`           |

## Tips

- Rare diseases often have few trials - use broad searches first
- Search by causative gene if disease-specific trials are limited
- Gene therapy trials are often early phase (Phase 1/2)
- Check for patient registries for natural history data
