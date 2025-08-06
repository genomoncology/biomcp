# PubTator3 API

This document describes the PubTator3 API used by BioMCP for searching biomedical literature and retrieving article details with annotations. Understanding this API provides context for how BioMCP's article commands function.

## Overview

The PubTator3 API provides a way to search for and retrieve biomedical articles
with entity annotations. This document outlines the API implementation details.
PubTator3 is a web-based tool that provides annotations of biomedical entities
in PubMed abstracts and PMC full-text articles. BioMCP uses the PubTator3 API
to search for and retrieve biomedical articles and their annotated entities (
genes, variants, diseases, chemicals, etc.).

> **CLI Documentation**: For information on using these APIs through the BioMCP
> command line interface, see
> the [Articles CLI Documentation](../user-guides/01-command-line-interface.md#article-commands).

## Usage Guide

For practical examples of searching articles with PubTator3, see [How to Find Articles and cBioPortal Data](../how-to-guides/01-find-articles-and-cbioportal-data.md).

## API Workflow

The PubTator3 integration follows a three-step workflow:

1. **Entity Autocomplete**: Get standardized entity identifiers
2. **Search**: Find articles using entity identifiers and keywords
3. **Fetch**: Retrieve full article details by PMID

## API Endpoints

### Entity Autocomplete API

**Endpoint:**
`https://www.ncbi.nlm.nih.gov/research/pubtator3-api/entity/autocomplete/`

This endpoint helps normalize entity names to their standard identifiers,
improving search precision.

#### Parameters

| Parameter | Description                 | Example                             |
| --------- | --------------------------- | ----------------------------------- |
| `query`   | Text to autocomplete        | `BRAF`                              |
| `concept` | Entity type                 | `GENE`, `CHEMICAL`, `DISEASE`, etc. |
| `limit`   | Number of results to return | `2`                                 |

#### Example Request and Response

```bash
curl "https://www.ncbi.nlm.nih.gov/research/pubtator3-api/entity/autocomplete/?query=BRAF&concept=GENE&limit=2"
```

Response:

```json
[
  {
    "_id": "@GENE_BRAF",
    "biotype": "gene",
    "name": "BRAF",
    "description": "All Species",
    "match": "Matched on name <m>BRAF</m>"
  },
  {
    "_id": "@GENE_BRAFP1",
    "biotype": "gene",
    "name": "BRAFP1",
    "description": "All Species",
    "match": "Matched on name <m>BRAFP1</m>"
  }
]
```

### Entity Search API

**Endpoint:** `https://www.ncbi.nlm.nih.gov/research/pubtator3-api/search/`

This endpoint allows searching for PMIDs (PubMed IDs) based on entity
identifiers and keywords.

#### Parameters

| Parameter | Description                     | Example                |
| --------- | ------------------------------- | ---------------------- |
| `text`    | Entity identifier or text query | `@CHEMICAL_remdesivir` |

#### Example Request and Response

```bash
curl "https://www.ncbi.nlm.nih.gov/research/pubtator3-api/search/?text=@CHEMICAL_remdesivir"
```

Response (truncated):

```json
{
  "results": [
    {
      "_id": "37711410",
      "pmid": 37711410,
      "title": "Remdesivir.",
      "journal": "Hosp Pharm",
      "authors": ["Levien TL", "Baker DE"],
      "date": "2023-10-01T00:00:00Z",
      "doi": "10.1177/0018578721999804",
      "meta_date_publication": "2023 Oct",
      "meta_volume": "58"
    }
    // More results...
  ]
}
```

### Article Fetch API

**Endpoint:**
`https://www.ncbi.nlm.nih.gov/research/pubtator3-api/publications/export/biocjson`

This endpoint retrieves detailed information about specific articles, including
annotations.

#### Parameters

| Parameter   | Description                                   | Example    |
| ----------- | --------------------------------------------- | ---------- |
| `pmids`     | List of PubMed IDs to retrieve                | `29355051` |
| `full_text` | Whether to include full text (when available) | `true`     |

#### Example Request

```bash
curl "https://www.ncbi.nlm.nih.gov/research/pubtator3-api/publications/export/biocjson?pmids=29355051&full=true"
```

Response format (truncated):

```json
{
  "PubTator3": [
    {
      "_id": "29355051|PMC6142073",
      "id": "6142073",
      "infons": {},
      "passages": [
        {
          "infons": {
            "name_3": "surname:Hu;given-names:Minghua",
            "name_2": "surname:Luo;given-names:Xia",
            "name_1": "surname:Luo;given-names:Shuang",
            "article-id_pmid": "29355051"
            // More metadata...
          }
        }
        // More passages...
      ]
    }
  ]
}
```

## Entity Types

PubTator3 annotates several types of biomedical entities:

1. **Genes/Proteins**: Gene or protein names (e.g., BRAF, TP53)
2. **Genetic Variants**: Genetic variations (e.g., BRAF V600E)
3. **Diseases**: Disease names and conditions (e.g., Melanoma)
4. **Chemicals/Drugs**: Chemical substances or drugs (e.g., Vemurafenib)

## Integration Strategy for BioMCP

The recommended workflow for integrating with PubTator3 in BioMCP is:

1. **Entity Normalization**: Use the autocomplete API to convert user-provided
   entity names to standardized identifiers
2. **Literature Search**: Use the search API with these identifiers to find
   relevant PMIDs
3. **Data Retrieval**: Fetch detailed article data with annotations using the
   fetch API

This workflow ensures consistent entity handling and optimal search results.

## Authentication

The PubTator3 API is public and does not require authentication for basic
usage. However, there are rate limits in place to prevent abuse.

## Rate Limits and Best Practices

- **Request Limits**: Approximately 30 requests per minute
- **Batch Requests**: For article retrieval, batch multiple PMIDs in a single
  request
- **Caching**: Implement caching to minimize repeated requests
- **Specific Queries**: Use specific entity names rather than general terms for
  better results

## Error Handling

Common error responses:

- **400**: Invalid parameters
- **404**: Articles not found
- **429**: Rate limit exceeded
- **500**: Server error

## More Information

For complete API documentation, visit
the [PubTator3 API Documentation](https://www.ncbi.nlm.nih.gov/research/pubtator3/api).
