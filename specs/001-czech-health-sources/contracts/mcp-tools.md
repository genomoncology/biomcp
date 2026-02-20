# MCP Tool Contracts: Czech Healthcare Data Sources

**Feature**: 001-czech-health-sources
**Date**: 2026-02-17

## Tool Registration Pattern

All Czech tools follow the BioMCP pattern:
```python
@mcp_app.tool()
@track_performance("czechmedmcp.{tool_name}")
async def tool_name(
    param: Annotated[type, Field(description="...")],
) -> str:
    ...
```

---

## SUKL Tools (5 MVP)

### sukl_drug_searcher

**Purpose**: Search Czech drug registry by name, substance, or ATC code

**Parameters**:

| Name | Type | Required | Description |
|------|------|----------|-------------|
| query | str | Yes | Drug name, active substance, or ATC code |
| page | int | No | Page number (1-based), default 1 |
| page_size | int | No | Results per page (1-100), default 10 |

**Returns**: JSON string with `total`, `page`, `page_size`, `results[]`
where each result has `sukl_code`, `name`, `active_substance`,
`atc_code`, `pharmaceutical_form`

**Errors**: Empty result set if no matches (not an error)

---

### sukl_drug_getter

**Purpose**: Get full drug details by SUKL code

**Parameters**:

| Name | Type | Required | Description |
|------|------|----------|-------------|
| sukl_code | str | Yes | SUKL drug identifier |

**Returns**: JSON string with full Drug entity fields

**Errors**: "Drug not found" if invalid SUKL code

---

### sukl_spc_getter

**Purpose**: Get Summary of Product Characteristics (SmPC)

**Parameters**:

| Name | Type | Required | Description |
|------|------|----------|-------------|
| sukl_code | str | Yes | SUKL drug identifier |

**Returns**: JSON string with `sukl_code`, `name`, `spc_text` (full SmPC
content), `spc_url`, `source`

**Errors**: "SmPC not available" if document not found

---

### sukl_pil_getter

**Purpose**: Get Patient Information Leaflet (PIL)

**Parameters**:

| Name | Type | Required | Description |
|------|------|----------|-------------|
| sukl_code | str | Yes | SUKL drug identifier |

**Returns**: JSON string with `sukl_code`, `name`, `pil_text` (full PIL
content), `pil_url`, `source`

**Errors**: "PIL not available" if document not found

---

### sukl_availability_checker

**Purpose**: Check current drug market availability

**Parameters**:

| Name | Type | Required | Description |
|------|------|----------|-------------|
| sukl_code | str | Yes | SUKL drug identifier |

**Returns**: JSON string with `sukl_code`, `name`, `status`
("available"/"limited"/"unavailable"), `last_checked`, `note`, `source`

**Errors**: "Drug not found" if invalid code

---

## MKN-10 Tools (3 MVP)

### mkn_diagnosis_searcher

**Purpose**: Search Czech ICD-10 diagnoses by code or text

**Parameters**:

| Name | Type | Required | Description |
|------|------|----------|-------------|
| query | str | Yes | MKN-10 code or free text in Czech |
| max_results | int | No | Maximum results, default 10 |

**Returns**: JSON string with `total`, `results[]` where each has
`code`, `name_cs`, `chapter`, `block`

**Errors**: Empty result set if no matches

---

### mkn_diagnosis_getter

**Purpose**: Get full diagnosis details including hierarchy

**Parameters**:

| Name | Type | Required | Description |
|------|------|----------|-------------|
| code | str | Yes | MKN-10 code (e.g., "J06.9") |

**Returns**: JSON string with full Diagnosis entity fields including
hierarchy, includes, excludes

**Errors**: "Invalid MKN-10 code" if code not found

---

### mkn_category_browser

**Purpose**: Browse MKN-10 category hierarchy

**Parameters**:

| Name | Type | Required | Description |
|------|------|----------|-------------|
| code | str | No | Category code to browse (omit for root/chapters) |

**Returns**: JSON string with `code`, `name`, `type`, `children[]`

**Errors**: "Category not found" if invalid code

---

## NRPZS Tools (2 MVP)

### nrpzs_provider_searcher

**Purpose**: Search healthcare providers by name, city, or specialty

**Parameters**:

| Name | Type | Required | Description |
|------|------|----------|-------------|
| query | str | No | Provider name or keyword |
| city | str | No | City name |
| specialty | str | No | Medical specialty |
| page | int | No | Page number, default 1 |
| page_size | int | No | Results per page, default 10 |

**Returns**: JSON string with `total`, `page`, `results[]` where each
has `provider_id`, `name`, `city`, `specialties[]`

**Errors**: At least one of query/city/specialty required

---

### nrpzs_provider_getter

**Purpose**: Get full provider details including workplaces

**Parameters**:

| Name | Type | Required | Description |
|------|------|----------|-------------|
| provider_id | str | Yes | NRPZS provider identifier |

**Returns**: JSON string with full HealthcareProvider entity fields

**Errors**: "Provider not found" if invalid ID

---

## SZV Tools (2 MVP)

### szv_procedure_searcher

**Purpose**: Search health procedures by code or name

**Parameters**:

| Name | Type | Required | Description |
|------|------|----------|-------------|
| query | str | Yes | Procedure code or name |
| max_results | int | No | Maximum results, default 10 |

**Returns**: JSON string with `total`, `results[]` where each has
`code`, `name`, `point_value`, `category`

**Errors**: Empty result set if no matches

---

### szv_procedure_getter

**Purpose**: Get full procedure details including point value

**Parameters**:

| Name | Type | Required | Description |
|------|------|----------|-------------|
| code | str | Yes | Procedure code (e.g., "09513") |

**Returns**: JSON string with full HealthProcedure entity fields

**Errors**: "Procedure not found" if invalid code

---

## VZP Tools (2 MVP)

### vzp_codebook_searcher

**Purpose**: Search VZP insurance codebooks

**Parameters**:

| Name | Type | Required | Description |
|------|------|----------|-------------|
| query | str | Yes | Search term |
| codebook_type | str | No | Filter by codebook type |
| max_results | int | No | Maximum results, default 10 |

**Returns**: JSON string with `total`, `results[]` where each has
`codebook_type`, `code`, `name`

**Errors**: Empty result set if no matches

---

### vzp_codebook_getter

**Purpose**: Get codebook entry details

**Parameters**:

| Name | Type | Required | Description |
|------|------|----------|-------------|
| codebook_type | str | Yes | Codebook type identifier |
| code | str | Yes | Entry code |

**Returns**: JSON string with full CodebookEntry entity fields

**Errors**: "Entry not found" if invalid codebook_type/code combination

---

## CLI Contracts

Each MCP tool has a corresponding CLI command under `biomcp czech`:

```
biomcp czech sukl search --query "Ibuprofen"
biomcp czech sukl get <sukl_code>
biomcp czech sukl spc <sukl_code>
biomcp czech sukl pil <sukl_code>
biomcp czech sukl availability <sukl_code>

biomcp czech mkn search --query "J06.9"
biomcp czech mkn get <code>
biomcp czech mkn browse [code]

biomcp czech nrpzs search --city "Praha" --specialty "kardiologie"
biomcp czech nrpzs get <provider_id>

biomcp czech szv search --query "EKG"
biomcp czech szv get <code>

biomcp czech vzp search --query "antibiotika"
biomcp czech vzp get <codebook_type> <code>
```

All CLI commands output JSON by default and support `--format human`
for readable output.
