# ClinicalTrials.gov API

This document outlines the key aspects of the public ClinicalTrials.gov v2 API utilized by BioMCP. Understanding these details can be helpful for advanced users interpreting BioMCP results or for developers extending its capabilities. BioMCP's CLI commands often simplify or combine these parameters for ease of use; refer to the [Trials CLI Documentation](../user-guides/01-command-line-interface.md#trial-commands) for specific command options.

## Overview

The [ClinicalTrials.gov](https://clinicaltrials.gov/) API provides programmatic
access to clinical trial information. This document outlines the API
implementation details for searching and retrieving clinical trial data.

> **CLI Documentation**: For information on using these APIs through the BioMCP
> command line interface, see the [Trials CLI Documentation](../user-guides/01-command-line-interface.md#trial-commands).

## API Endpoints

### Search API

**Endpoint:** `https://clinicaltrials.gov/api/v2/studies`

This endpoint allows searching for clinical trials using various parameters.

#### Key Parameters

| Parameter              | Description                         | Example Value                                   |
| ---------------------- | ----------------------------------- | ----------------------------------------------- |
| `query.cond`           | "Conditions or disease" query       | `lung cancer`                                   |
| `query.term`           | "Other terms" query                 | `AREA[LastUpdatePostDate]RANGE[2023-01-15,MAX]` |
| `query.intr`           | "Intervention/treatment" query      | `Vemurafenib`                                   |
| `query.locn`           | "Location terms" query              | `New York`                                      |
| `query.titles`         | "Title/acronym" query               | `BRAF Melanoma`                                 |
| `query.outc`           | "Outcome measure" query             | `overall survival`                              |
| `query.spons`          | "Sponsor/collaborator" query        | `National Cancer Institute`                     |
| `query.lead`           | Searches in "LeadSponsorName" field | `MD Anderson`                                   |
| `query.id`             | "Study IDs" query (OR semantics)    | `NCT04267848`                                   |
| `filter.overallStatus` | Comma-separated list of statuses    | `NOT_YET_RECRUITING,RECRUITING`                 |
| `filter.geo`           | Geo-location filter                 | `distance(39.0035707,-77.1013313,50mi)`         |
| `filter.ids`           | Filter by NCT IDs (AND semantics)   | `NCT04852770,NCT01728545`                       |
| `filter.advanced`      | Advanced filter query               | `AREA[StartDate]2022`                           |
| `sort`                 | Sort order                          | `LastUpdatePostDate:desc`                       |
| `fields`               | Fields to return                    | `NCTId,BriefTitle,OverallStatus,HasResults`     |

| `countTotal` | Count total number of studies | `true` or `false` |

#### Example Request

```bash
curl -X GET "https://clinicaltrials.gov/api/v2/studies?query.cond=Melanoma&query.intr=BRAF"
```

### Study Details API

**Endpoint:** `https://clinicaltrials.gov/api/v2/studies/{NCT_ID}`

This endpoint retrieves detailed information about a specific clinical trial.

#### Example Request

```bash
curl -X GET "https://clinicaltrials.gov/api/v2/studies/NCT04267848"
```

#### Response Modules

The API response contains various modules of information:

- **protocolSection**: Basic study information, eligibility criteria, and
  design
- **resultsSection**: Study outcomes and results (when available)
- **documentSection**: Related documents
- **derivedSection**: Derived data elements
- **annotationsSection**: Additional annotations

## Implementation Details

### NCT ID Filtering Semantics

BioMCP uses intelligent filtering when NCT IDs are provided:

- **ID-only mode**: When NCT IDs are the only filter criteria, `query.id` is used for fast direct lookup
- **Intersection mode**: When NCT IDs are combined with other filters (conditions, interventions, etc.), `filter.ids` is used to ensure results match ALL criteria

This ensures that specifying NCT IDs restricts results rather than expanding them.

### Query Building

When constructing API queries, parameters must be properly formatted according to the API documentation.

For implementation details on query building in BioMCP, see the [HTTP Client Developer Guide](../developer-guides/06-http-client-and-caching.md).

### Response Parsing

The API returns data in JSON format (or CSV if specified). Key sections in the
response include:

- `protocolSection`: Contains study protocol details
  - `identificationModule`: Basic identifiers including NCT ID and title
  - `statusModule`: Current recruitment status and study dates
  - `sponsorCollaboratorsModule`: Information about sponsors and
    collaborators
  - `designModule`: Study design information including interventions
  - `eligibilityModule`: Inclusion/exclusion criteria and eligible population
  - `contactsLocationsModule`: Study sites and contact information
  - `referencesModule`: Related publications

### Error Handling

The API returns standard HTTP status codes. Common error scenarios include:

- **404**: Trial not found
- **429**: Rate limit exceeded
- **400**: Invalid query parameters

For implementation details on error handling in BioMCP, see the [Error Handling Developer Guide](../developer-guides/05-error-handling.md).

## Authentication

The ClinicalTrials.gov API is public and does not require authentication for
basic usage. However, there are rate limits in place.

## Rate Limits and Best Practices

- **Rate Limit**: Approximately 50 requests per minute per IP address
- **Caching**: Implement caching to minimize repeated requests
- **Pagination**: For large result sets, use the pagination functionality with

- **Focused Queries**: Use specific search terms rather than broad queries to
  get more relevant results
- **Field Selection**: Use the fields parameter to request only the data you
  need

## More Information

For complete API documentation, visit
the [ClinicalTrials.gov API Documentation](https://clinicaltrials.gov/data-api/about-api)
