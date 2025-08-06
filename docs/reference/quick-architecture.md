# Quick Architecture Overview

## System Architecture (Simplified)

```
┌─────────────────────────────────────────────────────────────┐
│                        Clients                               │
├─────────────┬──────────────┬──────────────┬─────────────────┤
│     CLI     │    Claude    │  Python SDK  │  Custom Client  │
└──────┬──────┴──────┬───────┴──────┬───────┴─────────┬───────┘
       │             │              │                 │
       └─────────────┴──────────────┴─────────────────┘
                            │
                    ┌───────▼────────┐
                    │   BioMCP Core   │
                    │  (MCP Server)   │
                    └───────┬────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
┌───────▼────────┐ ┌────────▼────────┐ ┌───────▼────────┐
│ Article Handler │ │  Trial Handler  │ │ Variant Handler │
└───────┬────────┘ └────────┬────────┘ └───────┬────────┘
        │                   │                   │
┌───────▼────────┐ ┌────────▼────────┐ ┌───────▼────────┐
│ PubMed/PubTator│ │ ClinicalTrials  │ │  MyVariant.info │
│   cBioPortal   │ │    NCI CTS      │ │   AlphaGenome   │
└────────────────┘ └─────────────────┘ └────────────────┘
```

## Data Flow

```
User Query → Think → Plan → Search → Enrich → Format → Response
     │                        │         │                    │
     └────────────────────────┴─────────┴────────────────────┘
                          Cache Layer
```

## Quick Command Flow

```
$ biomcp article search --gene BRAF
         │
         ▼
    Parse Args → Validate → Route to Handler
                               │
                               ▼
                         Check Cache
                          Hit? │ Miss?
                           │   │
                           │   └→ Fetch from API → Store
                           │                         │
                           └─────────────────────────┘
                                       │
                                       ▼
                                Format & Return
```
