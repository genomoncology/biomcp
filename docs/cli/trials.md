# Trials CLI Documentation

The Trials CLI module provides commands for searching and retrieving clinical trial information from ClinicalTrials.gov.

> **API Documentation**: For details about the underlying API, see the [ClinicalTrials.gov API Documentation](../apis/clinicaltrials_gov.md).
>
> **Tip**: Use the `--help` flag with any command (e.g., `biomcp trial search --help`) to see the most up-to-date options directly from the tool.

## Search Command (`search`)

Search for clinical trials based on various filters and criteria.

### Usage

```bash
biomcp trial search [OPTIONS]
```

#### Basic Search Filters

- `-c, --condition TEXT`: Medical condition to search for (e.g., "Lung Cancer"). Can specify multiple times.
- `-i, --intervention TEXT`: Treatment or intervention to search for (e.g., "Pembrolizumab"). Can specify multiple times.
- `-t, --term TEXT`: General search terms (e.g., "immunotherapy"). Can specify multiple times.
- `-n, --nct-id TEXT`: Specific Clinical trial NCT ID(s) to look up (e.g., NCT04179552). Can specify multiple times.
  - **Note**: When NCT IDs are provided along with other filters (conditions, interventions, etc.), the results will be the intersection of both criteria - only trials that match ALL filters AND are in the NCT ID list will be returned. When NCT IDs are provided alone, only those specific trials are retrieved.

#### Study Characteristics Filters

- `-s, --status [OPEN|CLOSED|ANY]`: Filter by recruitment status. [default: OPEN]
- `--type [INTERVENTIONAL|OBSERVATIONAL|EXPANDED_ACCESS|OTHER]`: Filter by study type.
- `-p, --phase [EARLY_PHASE1|PHASE1|PHASE2|PHASE3|PHASE4|NOT_APPLICABLE]`: Filter by trial phase.
- `--purpose [TREATMENT|PREVENTION|DIAGNOSTIC|SUPPORTIVE_CARE|SCREENING|HEALTH_SERVICES|BASIC_SCIENCE|DEVICE_FEASIBILITY|OTHER]`: Filter by primary purpose.
- `-a, --age-group [CHILD|ADULT|SENIOR|ALL]`: Filter by participant age group. [default: ALL]

#### Advanced Filters

- `--min-date TEXT`: Minimum date for filtering (YYYY-MM-DD format). Requires `--date-field`.
- `--max-date TEXT`: Maximum date for filtering (YYYY-MM-DD format). Requires `--date-field`.
- `--date-field [LAST_UPDATE|STUDY_START|PRIMARY_COMPLETION|OUTCOME_POSTING|COMPLETION|FIRST_POSTING|SUBMITTED_DATE]`: Date field to use for filtering with `--min-date`/`--max-date`. [default: STUDY_START]
- `--intervention-type [DRUG|DEVICE|BIOLOGICAL|PROCEDURE|RADIATION|BEHAVIORAL|GENETIC|DIETARY|DIAGNOSTIC_TEST|OTHER]`: Filter by the type of intervention.
- `--sponsor-type [INDUSTRY|GOVERNMENT|ACADEMIC|OTHER]`: Filter by the type of sponsor.
- `--study-design [RANDOMIZED|NON_RANDOMIZED|OBSERVATIONAL]`: Filter by study design.

#### Location-based Search

- `--lat FLOAT`: Latitude for location-based search (requires `--lon` and `--distance`).
- `--lon FLOAT`: Longitude for location-based search (requires `--lat` and `--distance`).
- `-d, --distance INTEGER`: Distance in miles for location-based search (requires `--lat` and `--lon`).

#### Results Management

- `--sort [RELEVANCE|LAST_UPDATE|ENROLLMENT|START_DATE|COMPLETION_DATE|SUBMITTED_DATE]`: Sort order for results. [default: RELEVANCE]
- `-j, --json`: Render output in JSON format instead of Markdown.
- `--help`: Show help message and exit.

#### Examples

Search for clinical trials about melanoma (default status is OPEN):

```bash
biomcp trial search --condition "Melanoma"
```

Search for completed trials involving Vemurafenib:

```bash
biomcp trial search --intervention "Vemurafenib" --status CLOSED
```

Search for recently started Phase 3 trials for cancer, sorted by start date:

```bash
biomcp trial search --condition "Cancer" --phase PHASE3 --sort START_DATE
```

Search for recruiting trials near Boston, MA (approx. coordinates):

```bash
biomcp trial search --condition "Diabetes" --status OPEN --lat 42.36 --lon -71.05 --distance 50
```

Search for Phase 2 Immunotherapy trials for Breast Cancer, recruiting:

```bash
biomcp trial search --condition "Breast Cancer" --intervention "Immunotherapy" --phase PHASE2 --status OPEN
```

Get results as JSON:

```bash
biomcp trial search --condition "Melanoma" --json
```

## Get Command (`get`)

Retrieve detailed information about a specific clinical trial by its NCT ID and optionally select a specific module of information.

### Usage

```bash
biomcp trial get [OPTIONS] NCT_ID [MODULE]
```

#### Arguments

- `NCT_ID`: The NCT identifier for the clinical trial (e.g., NCT04267848). [required]
- `MODULE`: Optional module to retrieve. [default: Protocol]
  - `Protocol`: Core study information (ID, status, design, eligibility, etc.)
  - `Locations`: Contact and site location information.
  - `References`: Associated publications and references.
  - `Outcomes`: Outcome measures and results (if available).

#### Options

- `-j, --json`: Render output in JSON format instead of Markdown.
- `--help`: Show help message and exit.

#### Examples

Get basic protocol information for a trial:

```bash
biomcp trial get NCT04267848
```

or

```bash
biomcp trial get NCT04267848 Protocol
```

Get location information for a trial:

```bash
biomcp trial get NCT04267848 Locations
```

Get references and publications for a trial:

```bash
biomcp trial get NCT04267848 References
```

Get outcomes data for a trial:

```bash
biomcp trial get NCT04267848 Outcomes
```

Get protocol information as JSON:

```bash
biomcp trial get NCT04267848 Protocol --json
```
