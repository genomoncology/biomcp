# Feature: Agent Skills for BioMCP

## Summary

Add a `skills/` folder containing ready-to-use prompts and workflows for AI agents working with biomedical data. These skills enable LLM-based agents to effectively use BioMCP for clinical and research tasks.

## Motivation

BioMCP provides powerful CLI and MCP tools for querying biomedical databases, but users need guidance on:
- How to install and configure BioMCP
- Which commands to use for specific research questions
- How to chain multiple queries for comprehensive analysis

Agent skills solve this by providing:
1. A main `skills.md` file with installation and CLI overview
2. Curated use cases demonstrating real-world workflows
3. Copy-paste ready command examples

## Scope

### In Scope

- Main skills.md with BioMCP overview and installation
- 10 use case files covering precision medicine, clinical trials, pharmacovigilance
- Practical CLI examples that work out of the box
- Links between skills.md and use cases

### Out of Scope

- MCP server configuration (covered in main docs)
- API key setup details (covered in main docs)
- Programmatic Python SDK usage

## Deliverables

```
skills/
├── skills.md                     # Main entry point
└── use-cases/
    ├── 01-precision-oncology.md
    ├── 02-trial-matching.md
    ├── 03-drug-safety.md
    ├── 04-variant-interpretation.md
    ├── 05-rare-disease.md
    ├── 06-drug-shortages.md
    ├── 07-cell-therapy.md
    ├── 08-immunotherapy.md
    ├── 09-drug-labels.md
    └── 10-hereditary-cancer.md
```

## Success Criteria

- [ ] skills.md provides clear installation instructions
- [ ] Each use case has working CLI examples
- [ ] Use cases cover diverse biomedical domains
- [ ] Skills can be referenced by AI agents via file path or URL
