# BioMCP Pi Skill Installation

## Summary

The BioMCP skill has been successfully installed for Pi agent. This enables Pi to query biomedical databases for clinical trials, research articles, genetic variants, and FDA data.

## Installation Details

### Skill Location

The biomcp skill is installed at:

- **Global Pi skills:** `~/.pi/agent/skills/biomcp/`
- **Claude skills:** `~/.claude/skills/biomcp/` (already existed)

### Pi Configuration

A Pi settings file has been created at `~/.pi/settings.json`:

```json
{
  "skills": ["~/.claude/skills"],
  "enableSkillCommands": true
}
```

### BioMCP CLI

The BioMCP CLI tool has been installed globally via uv:

- **Package:** `biomcp-python` v0.7.2
- **Executable:** `~/.local/bin/biomcp`
- **Version:** 0.7.2

## Skill Capabilities

The biomcp skill provides access to:

### Data Sources

- **PubTator3/PubMed** - Peer-reviewed biomedical literature
- **ClinicalTrials.gov** - Clinical trial registry
- **MyVariant.info** - Genetic variant annotations
- **MyGene.info** - Gene information
- **MyChem.info** - Drug/chemical data
- **MyDisease.info** - Disease information
- **OpenFDA** - FDA regulatory and safety data

### Use Cases

- Literature search ("find papers about BRAF")
- Clinical trials ("recruiting trials for lung cancer")
- Gene information ("what does ERBB2 do")
- Variant interpretation ("is BRCA1 p.E1250K pathogenic")
- Drug investigation ("adverse events for pembrolizumab")
- Disease lookup ("what is Lynch syndrome")

## Usage

### In Pi

The skill is automatically available to Pi. When you ask about biomedical topics, Pi will automatically load and use the biomcp skill.

You can also invoke it directly with:

```
/skill:biomcp
```

### Command Line

```bash
# Gene information
biomcp gene get TP53

# Clinical trials
biomcp trial search --condition "lung cancer" --phase PHASE3

# Literature search
biomcp article search --gene BRAF --disease melanoma

# Variant search
biomcp variant search --gene TP53 --significance pathogenic

# Drug information
biomcp drug get pembrolizumab

# Adverse events
biomcp openfda adverse search --drug pembrolizumab

# For structured output
biomcp gene get TP53 --json
```

## Skill Structure

```
~/.pi/agent/skills/biomcp/
├── SKILL.md              # Main skill documentation
└── use-cases/            # Detailed workflow examples
    ├── 01-variant-to-treatment.md
    ├── 02-drug-investigation.md
    ├── 03-trial-matching.md
    ├── 04-rare-disease.md
    ├── 05-drug-shortages.md
    ├── 06-advanced-therapies.md
    ├── 07-hereditary-cancer.md
    └── 08-resistance.md
```

## Verification

To verify the installation:

```bash
# Check skill exists
ls -la ~/.pi/agent/skills/biomcp/

# Check biomcp CLI
biomcp --version

# Test biomcp functionality
biomcp gene get TP53 | head -20
```

## Troubleshooting

If biomcp is not found in PATH:

```bash
# Add to ~/.bashrc or ~/.zshrc:
export PATH="$HOME/.local/bin:$PATH"
```

Enable debug logging:

```bash
BIOMCP_LOG_LEVEL=DEBUG biomcp gene get TP53
```

Health check:

```bash
biomcp health check --verbose
```

## References

- BioMCP Documentation: https://biomcp.org
- BioMCP GitHub: https://github.com/genomoncology/biomcp
- Pi Skills Documentation: `~/.nvm/versions/node/v20.19.5/lib/node_modules/@mariozechner/pi-coding-agent/docs/skills.md`
