# BioMCP Data Flow Diagram

This document illustrates how BioMCP (Biomedical Model Context Protocol) works, showing the interaction between AI clients, the MCP server, domains, and external data sources.

## High-Level Architecture

```mermaid
graph TB
    subgraph "AI Client Layer"
        AI[AI Assistant<br/>e.g., Claude, GPT]
    end

    subgraph "MCP Server Layer"
        MCP[MCP Server<br/>router.py]
        SEARCH[search tool]
        FETCH[fetch tool]
    end

    subgraph "Domain Routing Layer"
        ROUTER[Query Router]
        PARSER[Query Parser]
        UNIFIED[Unified Query<br/>Language]
    end

    subgraph "Domain Handlers"
        ARTICLES[Articles Domain<br/>Handler]
        TRIALS[Trials Domain<br/>Handler]
        VARIANTS[Variants Domain<br/>Handler]
        THINKING[Thinking Domain<br/>Handler]
    end

    subgraph "External APIs"
        subgraph "Article Sources"
            PUBMED[PubTator3/<br/>PubMed]
            BIORXIV[bioRxiv/<br/>medRxiv]
            EUROPEPMC[Europe PMC]
        end

        subgraph "Clinical Data"
            CLINICALTRIALS[ClinicalTrials.gov]
        end

        subgraph "Variant Sources"
            MYVARIANT[MyVariant.info]
            TCGA[TCGA]
            KG[1000 Genomes]
            CBIO[cBioPortal]
        end
    end

    %% Connections
    AI -->|MCP Protocol| MCP
    MCP --> SEARCH
    MCP --> FETCH

    SEARCH --> ROUTER
    ROUTER --> PARSER
    PARSER --> UNIFIED

    ROUTER --> ARTICLES
    ROUTER --> TRIALS
    ROUTER --> VARIANTS
    ROUTER --> THINKING

    ARTICLES --> PUBMED
    ARTICLES --> BIORXIV
    ARTICLES --> EUROPEPMC
    ARTICLES -.->|Gene enrichment| CBIO

    TRIALS --> CLINICALTRIALS

    VARIANTS --> MYVARIANT
    MYVARIANT --> TCGA
    MYVARIANT --> KG
    VARIANTS --> CBIO

    THINKING -->|Internal| THINKING

    classDef clientClass fill:#e1f5fe,stroke:#01579b,stroke-width:2px
    classDef serverClass fill:#f3e5f5,stroke:#4a148c,stroke-width:2px
    classDef domainClass fill:#e8f5e9,stroke:#1b5e20,stroke-width:2px
    classDef apiClass fill:#fff3e0,stroke:#e65100,stroke-width:2px

    class AI clientClass
    class MCP,SEARCH,FETCH serverClass
    class ARTICLES,TRIALS,VARIANTS,THINKING domainClass
    class PUBMED,BIORXIV,EUROPEPMC,CLINICALTRIALS,MYVARIANT,TCGA,KG,CBIO apiClass
```

## Detailed Search Flow

```mermaid
sequenceDiagram
    participant AI as AI Client
    participant MCP as MCP Server
    participant Router as Query Router
    participant Domain as Domain Handler
    participant API as External API

    AI->>MCP: search(query="gene:BRAF AND disease:melanoma")
    MCP->>Router: Parse & route query

    alt Unified Query
        Router->>Router: Parse field syntax
        Router->>Router: Create routing plan

        par Search Articles
            Router->>Domain: Search articles (BRAF, melanoma)
            Domain->>API: PubTator3 API call
            API-->>Domain: Article results
            Domain->>API: cBioPortal enrichment
            API-->>Domain: Mutation data
        and Search Trials
            Router->>Domain: Search trials (melanoma)
            Domain->>API: ClinicalTrials.gov API
            API-->>Domain: Trial results
        and Search Variants
            Router->>Domain: Search variants (BRAF)
            Domain->>API: MyVariant.info API
            API-->>Domain: Variant results
        end
    else Domain-specific
        Router->>Domain: Direct domain search
        Domain->>API: Single API call
        API-->>Domain: Domain results
    else Sequential Thinking
        Router->>Domain: Process thought
        Domain->>Domain: Update session state
        Domain-->>Router: Thought response
    end

    Domain-->>Router: Formatted results
    Router-->>MCP: Aggregated results
    MCP-->>AI: Standardized response
```

## Search Tool Parameters

```mermaid
graph LR
    subgraph "Search Tool Input"
        PARAMS[Parameters]
        QUERY[query: string]
        DOMAIN[domain: article/trial/variant/thinking]
        GENES[genes: list]
        DISEASES[diseases: list]
        CONDITIONS[conditions: list]
        LAT[lat/long: coordinates]
        THOUGHT[thought parameters]
    end

    subgraph "Search Modes"
        MODE1[Unified Query Mode<br/>Uses 'query' param]
        MODE2[Domain-Specific Mode<br/>Uses domain + params]
        MODE3[Thinking Mode<br/>Uses thought params]
    end

    PARAMS --> MODE1
    PARAMS --> MODE2
    PARAMS --> MODE3
```

## Domain-Specific Data Sources

```mermaid
graph TD
    subgraph "Articles Domain"
        A1[PubTator3/PubMed<br/>- Published articles<br/>- Annotations]
        A2[bioRxiv/medRxiv<br/>- Preprints<br/>- Early research]
        A3[Europe PMC<br/>- Open access<br/>- Full text]
        A4[cBioPortal Integration<br/>- Auto-enrichment when genes specified<br/>- Mutation summaries & hotspots]
    end

    subgraph "Trials Domain"
        T1[ClinicalTrials.gov<br/>- Active trials<br/>- Trial details<br/>- Location search]
    end

    subgraph "Variants Domain"
        V1[MyVariant.info<br/>- Variant annotations<br/>- Clinical significance]
        V2[TCGA<br/>- Cancer variants<br/>- Somatic mutations]
        V3[1000 Genomes<br/>- Population frequency<br/>- Allele data]
        V4[cBioPortal<br/>- Cancer mutations<br/>- Hotspots]
    end

    A1 -.->|When genes present| A4
    A2 -.->|When genes present| A4
    A3 -.->|When genes present| A4
```

## Unified Query Language

```mermaid
graph TD
    QUERY[Unified Query<br/>"gene:BRAF AND disease:melanoma"]

    QUERY --> PARSE[Query Parser]

    PARSE --> F1[Field: gene<br/>Value: BRAF]
    PARSE --> F2[Field: disease<br/>Value: melanoma]

    F1 --> D1[Articles Domain]
    F1 --> D2[Variants Domain]
    F2 --> D1
    F2 --> D3[Trials Domain]

    D1 --> R1[PubMed Results]
    D2 --> R2[Variant Results]
    D3 --> R3[Trial Results]

    R1 --> AGG[Aggregated Results]
    R2 --> AGG
    R3 --> AGG
```

## Example: Location-Based Trial Search

```mermaid
sequenceDiagram
    participant User as User
    participant AI as AI Client
    participant MCP as BioMCP
    participant GEO as Geocoding Service
    participant CT as ClinicalTrials.gov

    User->>AI: Find active trials in Cleveland for NSCLC
    AI->>AI: Recognize location needs geocoding
    AI->>GEO: Geocode "Cleveland"
    GEO-->>AI: lat: 41.4993, long: -81.6944

    AI->>MCP: search(domain="trial",<br/>diseases=["NSCLC"],<br/>lat=41.4993,<br/>long=-81.6944,<br/>distance=50)

    MCP->>CT: API call with geo filter
    CT-->>MCP: Trials near Cleveland
    MCP-->>AI: Formatted trial results
    AI-->>User: Here are X active NSCLC trials in Cleveland area
```

## Key Features

1. **Parallel Execution**: Multiple domains are searched simultaneously for unified queries
2. **Smart Enrichment**: Article searches automatically include cBioPortal mutation summaries when genes are specified, providing clinical context alongside literature results
3. **Location Awareness**: Trial searches support geographic filtering with lat/long coordinates
4. **Sequential Thinking**: Built-in reasoning system for complex biomedical questions
5. **Standardized Output**: All results follow OpenAI MCP format for consistency

## Response Format

All search results follow this standardized structure:

```json
{
  "results": [
    {
      "id": "PMID12345678",
      "title": "BRAF V600E mutation in melanoma",
      "text": "This study investigates BRAF mutations...",
      "url": "https://pubmed.ncbi.nlm.nih.gov/12345678"
    }
  ]
}
```

Fetch results include additional domain-specific metadata in the response.
