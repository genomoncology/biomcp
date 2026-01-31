# Skills CLI Redesign

Redesign of the `biomcp skill` command to provide a self-documenting, discoverable interface for agent skills.

## Command Structure

```
biomcp skill                    # Show SKILL.md + CLI usage
biomcp skills                   # Alias (both singular/plural work)
biomcp skill list               # List available use-cases
biomcp skill install [dir]      # Install skills to directory
biomcp skill <name-or-number>   # Show specific use-case
```

## Subcommands

### `biomcp skill` (default)

Display the main SKILL.md content with CLI usage appended.

**Output:**

```
[Contents of SKILL.md]

---
## CLI Usage

View skills:
  biomcp skill              Show this help
  biomcp skill list         List use-case patterns
  biomcp skill <name>       Show specific use-case (by name or number)

Install skills:
  biomcp skill install      Auto-detect agent and prompt
  biomcp skill install <dir>  Install to specific directory

Examples:
  biomcp skill 01                    # Show 01-variant-to-treatment
  biomcp skill variant-to-treatment  # Same (without number prefix)
  biomcp skill install ~/.claude     # Install for Claude Code
```

The CLI usage section is appended dynamically (HATEOAS-style) when viewing via CLI, not stored in SKILL.md itself.

### `biomcp skill list`

Show available use-cases in compact format.

**Output:**

```
Available use-cases:

  01  variant-to-treatment   Variant → interpretation → trials → treatment
  02  drug-investigation     Drug → adverse events → labels → approvals
  03  trial-matching         Patient criteria → filtered trial search
  04  rare-disease           Rare condition → gene therapy → registries
  05  drug-shortages         Drug → shortage status → alternatives
  06  advanced-therapies     CAR-T, immunotherapy trial landscape
  07  hereditary-cancer      Syndrome → genes → surveillance trials
  08  resistance             Prior therapy → resistance → next-line options

View details: biomcp skill <number-or-name>
```

### `biomcp skill <name-or-number>`

Display a specific use-case file.

**Resolution rules:**

1. Number match: `01` → `01-variant-to-treatment.md`
2. Exact name: `variant-to-treatment` → `01-variant-to-treatment.md`
3. Full filename: `01-variant-to-treatment` → `01-variant-to-treatment.md`

**Errors:**

- No match: "Use-case 'foo' not found. Run 'biomcp skill list' to see available options."
- Ambiguous: Not possible with number-or-exact-name matching.

### `biomcp skill install [directory]`

Install skills to the specified directory.

#### No directory specified: Auto-detect and prompt

Check these directories in order:

1. `~/.claude/skills/`
2. `~/.codex/skills/`
3. `~/.config/opencode/skills/`
4. `~/.gemini/skills/`
5. `~/.pi/skills/`

For each that exists (parent directory exists), prompt:

```
Found Claude Code at ~/.claude/
Install BioMCP skills to ~/.claude/skills/biomcp? [Y/n]
```

If none found:

```
No known agent directories found.
Specify a directory: biomcp skill install <directory>

Common locations:
  ~/.claude/skills/     Claude Code
  ~/.codex/skills/      OpenAI Codex
  ~/.pi/skills/         Pi Agent
```

#### Directory specified: Parse and confirm

**Path parsing rules:**

| Input                     | Resolved Path             | Logic                                 |
| ------------------------- | ------------------------- | ------------------------------------- |
| `~/.claude`               | `~/.claude/skills/biomcp` | No `skills` → append `skills/biomcp`  |
| `~/.claude/skills`        | `~/.claude/skills/biomcp` | Ends with `skills` → append `biomcp`  |
| `~/.claude/skills/biomcp` | `~/.claude/skills/biomcp` | Ends with `skills/<name>` → use as-is |
| `~/.claude/skills/custom` | `~/.claude/skills/custom` | Ends with `skills/<name>` → use as-is |
| `./.claude/skills`        | `./.claude/skills/biomcp` | Relative paths work too               |

**Detection logic:**

```python
parts = path.parts
if len(parts) >= 2 and parts[-2] == "skills":
    # Path is skills/<something>, use as-is
    target = path
elif parts[-1] == "skills":
    # Path ends in skills/, append biomcp
    target = path / "biomcp"
else:
    # No skills in path, append skills/biomcp
    target = path / "skills" / "biomcp"
```

**Confirmation prompt:**

```
Install BioMCP skills to ~/.claude/skills/biomcp? [Y/n]
```

#### Existing installation check

If target directory already exists:

```
BioMCP skills already installed at ~/.claude/skills/biomcp/

To replace: biomcp skill install ~/.claude --force
To view:    biomcp skill
```

#### `--force` flag

Delete existing installation and replace entirely.

```
Replacing existing BioMCP skills at ~/.claude/skills/biomcp/
Done.
```

## Files to Change

### Remove

- `src/biomcp/cli/skills.py` (old install-skill command)

### Create

- `src/biomcp/cli/skill.py` (new skill command with subcommands)

### Modify

- `src/biomcp/cli/main.py` - Register new `skill` command, remove old `install-skill`

## Implementation Notes

### Skill content loading

```python
SKILLS_SOURCE = Path(__file__).parent.parent / "skills" / "biomcp"
SKILL_MD = SKILLS_SOURCE / "SKILL.md"
USE_CASES_DIR = SKILLS_SOURCE / "use-cases"
```

### CLI usage suffix

Appended when displaying SKILL.md:

```python
CLI_USAGE_SUFFIX = """
---
## CLI Usage

View skills:
  biomcp skill              Show this help
  biomcp skill list         List use-case patterns
  biomcp skill <name>       Show specific use-case (by name or number)

Install skills:
  biomcp skill install      Auto-detect agent and prompt
  biomcp skill install <dir>  Install to specific directory

Examples:
  biomcp skill 01           Show 01-variant-to-treatment
  biomcp skill install      Guided installation
"""
```

### Use-case index building

```python
def build_use_case_index():
    """Build index of use-cases for resolution."""
    index = {}
    for f in USE_CASES_DIR.glob("*.md"):
        name = f.stem  # e.g., "01-variant-to-treatment"
        number = name.split("-")[0]  # e.g., "01"
        rest = "-".join(name.split("-")[1:])  # e.g., "variant-to-treatment"

        index[name] = f          # Full name
        index[number] = f        # Number only
        index[rest] = f          # Name without number
    return index
```

### Agent directory detection

```python
KNOWN_AGENTS = [
    ("Claude Code", Path.home() / ".claude"),
    ("OpenAI Codex", Path.home() / ".codex"),
    ("OpenCode", Path.home() / ".config" / "opencode"),
    ("Gemini CLI", Path.home() / ".gemini"),
    ("Pi Agent", Path.home() / ".pi"),
]

def detect_agents():
    """Find installed agents by checking for config directories."""
    found = []
    for name, path in KNOWN_AGENTS:
        if path.exists():
            found.append((name, path / "skills" / "biomcp"))
    return found
```

## Example Sessions

### First-time install (auto-detect)

```
$ biomcp skill install

Found Claude Code at ~/.claude/
Install BioMCP skills to ~/.claude/skills/biomcp? [Y/n] y

Installed BioMCP skills to ~/.claude/skills/biomcp/
View with: biomcp skill
```

### Install to specific directory

```
$ biomcp skill install ~/.claude

Install BioMCP skills to ~/.claude/skills/biomcp? [Y/n] y

Installed BioMCP skills to ~/.claude/skills/biomcp/
```

### Already installed

```
$ biomcp skill install ~/.claude

BioMCP skills already installed at ~/.claude/skills/biomcp/

To replace: biomcp skill install ~/.claude --force
To view:    biomcp skill
```

### View use-case

```
$ biomcp skill 01

# Pattern: Variant to Treatment

From detected variant to actionable treatment options.
...
```

### List use-cases

```
$ biomcp skill list

Available use-cases:

  01  variant-to-treatment   Variant → interpretation → trials → treatment
  02  drug-investigation     Drug → adverse events → labels → approvals
  ...

View details: biomcp skill <number-or-name>
```
