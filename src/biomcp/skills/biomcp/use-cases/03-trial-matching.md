# Pattern: Clinical Trial Matching

Find eligible trials for a patient with specific characteristics.

## Workflow

1. **Normalize disease** - `biomcp disease search <term>` to get standard terminology
2. **Search by condition** - `biomcp trial search --condition <disease>`
3. **Add molecular criteria** - `--biomarker <gene>` for mutation-specific trials
4. **Filter by status** - `--status RECRUITING` for enrolling trials
5. **Refine by phase/location** - `--phase PHASE3`, geographic filters

## Status Values

Use UPPERCASE: `OPEN`, `RECRUITING`, `ACTIVE_NOT_RECRUITING`, `COMPLETED`

## Useful Filters

| Filter                   | Purpose                            |
| ------------------------ | ---------------------------------- |
| `--phase`                | PHASE1, PHASE2, PHASE3             |
| `--intervention`         | Drug or therapy name               |
| `--biomarker`            | Required genetic marker            |
| `--age-group`            | child, adult, older_adult          |
| `--required-mutation`    | Trials requiring specific mutation |
| `--line-of-therapy`      | 1L, 2L, 3L+ treatment line         |
| `--lat/--lon/--distance` | Geographic search (miles)          |

## Tips

- Use `biomcp biomarker search` to find trials by eligibility biomarkers
- Use `biomcp organization search` to find trial sites
- Cross-reference variant data for molecular eligibility
