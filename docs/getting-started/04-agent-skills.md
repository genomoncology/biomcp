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
biomcp install-skill <your-agent-skills-directory>
```

Examples by agent:

| Agent | Command |
|-------|---------|
| Claude Code | `biomcp install-skill ~/.claude/skills/` |
| OpenAI Codex | `biomcp install-skill ~/.codex/skills/` |
| OpenCode | `biomcp install-skill ~/.opencode/skills/` |
| Gemini CLI | `biomcp install-skill ~/.gemini/skills/` |
| Pi Agent | `biomcp install-skill ~/.pi/skills/` |
| Project-local | `biomcp install-skill ./.claude/skills/` |

## Available Skills

The BioMCP skill teaches agents how to query biomedical databases via CLI. It includes these workflow patterns:

| Skill | Description |
|-------|-------------|
| Mutation to Trial | Mutation → annotations → trials → literature |
| Drug Investigation | Drug → adverse events → approvals → labels |
| Trial Matching | Patient criteria → filtered trial search |
| Variant Interpretation | Variant → pathogenicity → actionability |
| Rare Disease Research | Rare condition → gene therapy → registries |
| Drug Shortages | Drug → shortage status → alternatives |
| Cell Therapy | CAR-T/TIL/TCR-T trial landscape |
| Immunotherapy | Biomarker-driven checkpoint inhibitor trials |
| Drug Labels | FDA labels → indications → warnings |
| Hereditary Cancer | Syndrome → genes → surveillance trials |
| Resistance Research | Prior therapy → resistance → next-line options |

## Quick Reference

Once installed, agents can use these commands:

| Command | Purpose |
|---------|---------|
| `biomcp article search` | Find literature (PubMed) |
| `biomcp article get` | Get article by PMID |
| `biomcp trial search` | Find clinical trials |
| `biomcp trial get` | Get trial by NCT ID |
| `biomcp variant search` | Query ClinVar variants |
| `biomcp variant get` | Get variant by rsID |
| `biomcp openfda adverse search` | Drug adverse events |
| `biomcp openfda approval search` | Drug approvals |
| `biomcp openfda shortage search` | Drug shortages |
| `biomcp openfda label search` | Drug labels |

**Discover all options:** `biomcp <command> --help`

## Key Patterns

- Use `-j` or `--json` for structured output
- Normalize genes to HGNC symbols (HER2 → ERBB2)
- Normalize variants to HGVS notation (V600E → p.Val600Glu)
- Status enums are UPPERCASE (OPEN, RECRUITING, PHASE3)

## Example Queries

After installation, try asking your agent:

- "Is BRAF V600E pathogenic? Find recruiting trials for melanoma."
- "What serious adverse events are reported for pembrolizumab?"
- "Find Phase 3 CAR-T trials for relapsed lymphoma."
- "A patient has Lynch syndrome (MLH1 variant). What cancers are associated and are there prevention trials?"
- "Research resistance mechanisms for dabrafenib + trametinib in melanoma."
