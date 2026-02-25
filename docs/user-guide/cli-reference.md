# CLI Reference

BioMCP provides one command family with entity-oriented subcommands.

## Global options

- `--json`: return structured JSON output
- `--no-cache`: bypass HTTP cache for the current command

## Core command patterns

```text
biomcp search <entity> [filters]
biomcp get <entity> <id> [section...]
```

Section names are positional trailing arguments after `<id>`.

## Top-level commands

```text
biomcp search ...
biomcp get ...
biomcp enrich <GENE1,GENE2,...> [--limit N]
biomcp batch <entity> <id1,id2,...> [--sections ...] [--source ...]
biomcp health [--apis-only]
biomcp list [entity]
biomcp skill [list|install|<name>]
biomcp mcp
biomcp serve
biomcp serve-http [--host 127.0.0.1] [--port 8080]
biomcp update [--check]
biomcp uninstall
biomcp version
```

## Search command families

### Gene

```bash
biomcp search gene -q BRAF --limit 10 --offset 0
```

### Disease

```bash
biomcp search disease -q melanoma --source mondo --limit 10 --offset 0
```

### PGx

```bash
biomcp search pgx -g CYP2D6 --limit 10
biomcp search pgx -d warfarin --limit 10
```

### Phenotype (Monarch semsim)

```bash
biomcp search phenotype "HP:0001250 HP:0001263" --limit 10
```

### GWAS

```bash
biomcp search gwas -g TCF7L2 --limit 10
biomcp search gwas --trait "type 2 diabetes" --limit 10
```

### Article

```bash
biomcp search article -g BRAF -d melanoma --since 2024-01-01 --limit 5 --offset 0
```

### Trial

```bash
biomcp search trial -c melanoma --status recruiting --source ctgov --limit 5 --offset 0
```

### Variant

```bash
biomcp search variant -g BRAF --hgvsp V600E --limit 5 --offset 0
```

### Drug

```bash
biomcp search drug -q "kinase inhibitor" --limit 5 --offset 0
```

### Pathway

```bash
biomcp search pathway -q "MAPK signaling" --limit 5 --offset 0
```

### Protein

```bash
biomcp search protein -q kinase --limit 5 --offset 0
biomcp search protein -q kinase --all-species --limit 5
```

### Adverse event

```bash
biomcp search adverse-event --drug pembrolizumab --serious --limit 5 --offset 0
biomcp search adverse-event --type device --manufacturer Medtronic --limit 5
biomcp search adverse-event --type device --product-code PQP --limit 5
```

## Get command families

### Gene

```bash
biomcp get gene BRAF
biomcp get gene BRAF pathways
biomcp get gene BRAF diseases
```

### Disease

```bash
biomcp get disease melanoma
biomcp get disease MONDO:0005105 genes phenotypes
biomcp get disease MONDO:0005105 variants models
biomcp get disease MONDO:0005105 pathways prevalence civic
biomcp get disease MONDO:0005105 all
```

### PGx

```bash
biomcp get pgx CYP2D6
biomcp get pgx codeine recommendations frequencies
biomcp get pgx warfarin annotations
```

### Article

```bash
biomcp get article 22663011
biomcp get article 22663011 fulltext
```

### Trial

```bash
biomcp get trial NCT02576665
biomcp get trial NCT02576665 eligibility
```

### Variant

```bash
biomcp get variant "BRAF V600E"
biomcp get variant "BRAF V600E" predict
biomcp get variant rs7903146 gwas
```

### Drug

```bash
biomcp get drug pembrolizumab
biomcp get drug carboplatin shortage
```

### Pathway

```bash
biomcp get pathway R-HSA-5673001
biomcp get pathway R-HSA-5673001 genes
```

### Protein

```bash
biomcp get protein P15056
biomcp get protein P15056 domains interactions
```

### Adverse event

```bash
biomcp get adverse-event 10222779
biomcp get adverse-event 10222779 reactions outcomes
biomcp get adverse-event 10222779 concomitant guidance all
```

## Enrichment

```bash
biomcp enrich BRAF,KRAS,NRAS --limit 10
biomcp enrich BRAF,KRAS,NRAS --limit 10 --json
```

## MCP mode

- `biomcp serve` runs the stdio MCP server.
- `biomcp serve-http` runs MCP over HTTP/SSE.

See also: `docs/reference/mcp-server.md`.

## Helper command families

```bash
biomcp variant trials "BRAF V600E"
biomcp variant articles "BRAF V600E"
biomcp variant oncokb "BRAF V600E"
biomcp drug adverse-events pembrolizumab
biomcp drug trials pembrolizumab
biomcp disease trials melanoma
biomcp disease drugs melanoma
biomcp disease articles "Lynch syndrome"
biomcp gene trials BRAF
biomcp gene drugs BRAF
biomcp gene articles BRCA1
biomcp gene pathways BRAF
biomcp pathway drugs R-HSA-5673001
biomcp pathway articles R-HSA-5673001
biomcp pathway trials R-HSA-5673001
biomcp protein structures P15056
biomcp article entities 22663011
```

## Batch mode

```bash
biomcp batch gene BRAF,TP53
biomcp batch gene BRAF,TP53 --sections pathways,interactions
biomcp batch trial NCT02576665,NCT03715933 --source nci
biomcp batch variant "BRAF V600E","KRAS G12D" --json
```
