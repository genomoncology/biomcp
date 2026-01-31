# BioMCP for AI Agents

Give this page to your AI coding agent and it will install BioMCP and the skills for biomedical research.

## Installation

### Step 1: Install BioMCP CLI

=== "uv (Recommended)"

    ```bash
    uv tool install biomcp-python
    ```

=== "pip"

    ```bash
    pip install biomcp-python
    ```

Verify installation:

```bash
biomcp --version
```

### Step 2: Install Agent Skills

```bash
biomcp skill install [directory]
```

Examples by agent:

| Agent         | Command                                   |
| ------------- | ----------------------------------------- |
| Claude Code   | `biomcp skill install ~/.claude`          |
| OpenAI Codex  | `biomcp skill install ~/.codex`           |
| OpenCode      | `biomcp skill install ~/.config/opencode` |
| Gemini CLI    | `biomcp skill install ~/.gemini`          |
| Pi Agent      | `biomcp skill install ~/.pi`              |
| Project-local | `biomcp skill install ./.claude`          |

## Available Skills

The BioMCP skill teaches agents how to query biomedical databases via CLI. It includes these workflow patterns:

| Skill                | Description                                    |
| -------------------- | ---------------------------------------------- |
| Variant to Treatment | Variant → interpretation → trials → treatment  |
| Drug Investigation   | Drug → adverse events → labels → approvals     |
| Trial Matching       | Patient criteria → filtered trial search       |
| Rare Disease         | Rare condition → gene therapy → registries     |
| Drug Shortages       | Drug → shortage status → alternatives          |
| Advanced Therapies   | CAR-T, immunotherapy trial landscape           |
| Hereditary Cancer    | Syndrome → genes → surveillance trials         |
| Resistance           | Prior therapy → resistance → next-line options |

## Quick Reference

Once installed, agents can use these commands:

| Command                          | Purpose                         |
| -------------------------------- | ------------------------------- |
| `biomcp article search`          | Search articles (PubTator3)     |
| `biomcp article get`             | Get article by PMID             |
| `biomcp trial search`            | Find clinical trials            |
| `biomcp trial get`               | Get trial by NCT ID             |
| `biomcp gene search/get`         | Gene info (MyGene.info)         |
| `biomcp variant search/get`      | Variant annotations (MyVariant) |
| `biomcp variant predict`         | Variant effects (AlphaGenome)   |
| `biomcp drug search/get`         | Drug info (MyChem.info)         |
| `biomcp disease search/get`      | Disease info (MyDisease.info)   |
| `biomcp biomarker search`        | Trial eligibility biomarkers    |
| `biomcp openfda adverse search`  | Drug adverse events (FAERS)     |
| `biomcp openfda approval search` | Drug approvals                  |
| `biomcp openfda shortage search` | Drug shortages                  |
| `biomcp openfda label search`    | Drug labels (SPL)               |
| `biomcp openfda device search`   | Device adverse events (MAUDE)   |
| `biomcp openfda recall search`   | Drug recalls                    |

**Discover all options:** `biomcp <command> --help`

## Key Patterns

- Use `-j` or `--json` for structured output
- Normalize genes to HGNC symbols (HER2 → ERBB2)
- Normalize variants to HGVS notation (V600E → p.Val600Glu)
- Status enums are UPPERCASE (OPEN, RECRUITING, PHASE3)
- Article search: `-g/--gene`, `-v/--variant`, `-d/--disease`, `-c/--chemical`
- Gene enrichment: `biomcp gene get <symbol> --enrich pathway`

## Example Queries

After installation, try asking your agent:

- "Is BRAF V600E pathogenic? Find recruiting trials for melanoma."
- "What serious adverse events are reported for pembrolizumab?"
- "Find Phase 3 CAR-T trials for relapsed lymphoma."
- "A patient has Lynch syndrome (MLH1 variant). What cancers are associated and are there prevention trials?"
- "Research resistance mechanisms for dabrafenib + trametinib in melanoma."
