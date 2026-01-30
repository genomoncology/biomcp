# BioMCP Bug Report

This document tracks bugs discovered during CLI exploration on 2026-01-30.

---

## Issue 1: cBioPortal Returns Wrong Gene Hotspots

**Severity:** HIGH
**Component:** `src/biomcp/variants/cbioportal_search_helpers.py`

### Description

When searching for variant hotspots by gene, the cBioPortal summary displays mutations from **wrong genes**. For example:

- Searching **KRAS** shows "V600E" (a BRAF mutation)
- Searching **TP53** shows "E545K" (a PIK3CA mutation)
- Searching **EGFR** shows "R132H" (an IDH1 mutation)

This produces misleading clinical data that could confuse researchers.

### Root Cause

The `_update_hotspot_counts()` function in `cbioportal_search_helpers.py` (lines 71-87) counts all `proteinChange` values from API responses without validating the `hugoGeneSymbol` field matches the requested gene.

```python
# Current code (lines 77-86):
for mut in mutations:
    protein_change = mut.get("proteinChange", "")
    if protein_change:
        # BUG: No validation that mutation belongs to requested gene
        hotspot_counts[protein_change]["count"] += 1
```

The cBioPortal API may return cross-gene data in some responses, and BioMCP does not filter it.

### Files Affected

- `src/biomcp/variants/cbioportal_search_helpers.py` - lines 71-87, 89-97
- `src/biomcp/variants/cbioportal_search.py` - lines 302-341

### Requirements

1. Add `hugoGeneSymbol` validation when processing mutation responses
2. Filter out mutations where gene symbol does not match the requested gene
3. Pass the requested gene symbol through the call chain to helper functions
4. Add test coverage for cross-gene contamination scenarios

### Reproduction

```bash
biomcp variant search --gene KRAS --significance pathogenic --size 5
# Observe: cBioPortal summary shows "V600E" which is BRAF, not KRAS
```

---

## Issue 2: Year-Based Drug Approval Search Broken

**Severity:** MEDIUM
**Component:** `src/biomcp/openfda/drug_approvals.py`

### Description

Searching for FDA drug approvals by year always returns "No matches found" regardless of year specified.

```bash
biomcp openfda approval search --year 2021 --limit 3
# Returns: "No matches found!"
```

### Root Cause

Two bugs in `drug_approvals.py` lines 56-60:

1. **Wrong field name:** Code searches `products.marketing_status_date` which doesn't exist in the API. Products only have `marketing_status` (no date field). The correct field is `submissions.submission_status_date`.

2. **Wrong date format:** Code uses `YYYY-MM-DD` format but OpenFDA expects `YYYYMMDD`.

```python
# Current code (line 58-60):
search_params["search"] = (
    f"products.marketing_status_date:[{approval_year}-01-01 TO {approval_year}-12-31]"
)
# Field doesn't exist, and dates have wrong format
```

### Files Affected

- `src/biomcp/openfda/drug_approvals.py` - lines 56-60

### Requirements

1. Change field from `products.marketing_status_date` to `submissions.submission_status_date`
2. Change date format from `YYYY-MM-DD` to `YYYYMMDD`
3. Verify query works against live OpenFDA API
4. Add test coverage for year-based searches

### Correct Query Format

```python
search_params["search"] = (
    f"submissions.submission_status_date:[{approval_year}0101 TO {approval_year}1231]"
)
```

### Reproduction

```bash
biomcp openfda approval search --year 2021 --limit 3
# Expected: List of 2021 drug approvals
# Actual: "No matches found!"
```

---

## Issue 3: Drug Shortage API Returns PDF Instead of Data

**Severity:** MEDIUM
**Component:** `src/biomcp/openfda/drug_shortages.py`

### Description

The drug shortage feature fails with a parsing error because the configured URL returns a PDF file, not JSON data.

```bash
biomcp openfda shortage search --status current --limit 10
# Returns: "Drug Shortage Data Temporarily Unavailable"
# Log shows: "Failed to parse response: new-line character seen in unquoted field"
```

### Root Cause

The code at line 46 points to a URL that returns a PDF:

```python
FDA_SHORTAGES_JSON_URL = "https://www.fda.gov/media/169066/download"  # Returns PDF!
```

However, FDA provides a **working CSV endpoint** with full shortage data:

```
https://www.accessdata.fda.gov/scripts/drugshortages/Drugshortages.cfm
```

This CSV contains all fields needed: Generic Name, Company, Status, Reason, Therapeutic Category, Availability, Dates, etc.

### Files Affected

- `src/biomcp/openfda/drug_shortages.py` - lines 42-46, 54-89
- `src/biomcp/openfda/drug_shortages_helpers.py`

### Requirements

1. Change URL from PDF endpoint to CSV endpoint
2. Parse CSV response using Python's `csv` module
3. Convert parsed data to internal format for filtering/display
4. For CLI output, display as CSV code block in markdown:
   ```csv
   Generic Name,Status,Reason,...
   Albuterol Sulfate Solution,Current,Demand increase,...
   ```
5. For MCP JSON API, convert CSV rows to JSON objects
6. Update cache logic to handle CSV format
7. Add test coverage for CSV parsing

### CSV Endpoint Details

**URL:** `https://www.accessdata.fda.gov/scripts/drugshortages/Drugshortages.cfm`

**Returns:** CSV with columns:
- Generic Name, Company Name, Contact Info, Presentation
- Type of Update, Date of Update, Availability Information
- Related Information, Resolved Note, Reason for Shortage
- Therapeutic Category, Status, Change Date
- Date Discontinued, Initial Posting Date

### Reproduction

```bash
biomcp openfda shortage search --status current --limit 10
# Expected: List of current drug shortages
# Actual: Error message about data unavailability

# Verify CSV endpoint works:
curl -s "https://www.accessdata.fda.gov/scripts/drugshortages/Drugshortages.cfm" | head -5
```

---

## Summary

| Issue | Severity | Type | Effort |
|-------|----------|------|--------|
| cBioPortal wrong hotspots | HIGH | Data validation | Medium |
| Year approval search | MEDIUM | Query format | Low |
| Drug shortage CSV | MEDIUM | Data source | Medium |
