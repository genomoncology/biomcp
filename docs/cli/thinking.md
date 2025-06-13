# Thinking CLI Commands

The BioMCP CLI includes sequential thinking commands for structured problem-solving from the command line.

## Overview

The thinking commands help you break down complex problems into manageable steps, track your reasoning process, and explore alternative approaches.

## Commands

### `biomcp thinking add`

Add a new thought to the sequential thinking process.

```bash
biomcp thinking add "First, let's analyze the problem" --number 1 --total 5
```

Options:
- `--number, -n`: Thought number in sequence (required)
- `--total, -t`: Total estimated thoughts needed (required)
- `--next`: Whether another thought is needed after this one
- `--revision`: Mark this as a revision of a previous thought
- `--revises`: Which thought number this revises
- `--branch`: Create a new branch from a specific thought
- `--branch-id`: Identifier for the new branch
- `--need-more`: Indicate more thoughts may be needed beyond total

### `biomcp thinking history`

View all recorded thoughts in the current thinking process.

```bash
biomcp thinking history
biomcp thinking history --json  # JSON format output
```

### `biomcp thinking branches`

View thoughts organized by branches.

```bash
biomcp thinking branches
biomcp thinking branches --id alternative-approach  # View specific branch
biomcp thinking branches --json  # JSON format output
```

### `biomcp thinking summary`

Get a summary of the current thinking process.

```bash
biomcp thinking summary
```

### `biomcp thinking clear`

Clear all thinking history and branches.

```bash
biomcp thinking clear --yes  # Confirm clearing
```

### `biomcp thinking guide`

Display a guide on using sequential thinking effectively.

```bash
biomcp thinking guide
```

## Examples

### Basic Sequential Thinking

```bash
# Start a new thinking process
biomcp thinking add "First, identify the key requirements" -n 1 -t 4 --next

# Continue with next steps
biomcp thinking add "Second, analyze available options" -n 2 -t 4 --next
biomcp thinking add "Third, evaluate pros and cons" -n 3 -t 4 --next
biomcp thinking add "Finally, make a recommendation" -n 4 -t 4

# View the complete thought history
biomcp thinking history
```

### Revising Previous Thoughts

```bash
# Add initial thought
biomcp thinking add "Use approach A for this problem" -n 1 -t 3 --next

# Revise based on new information
biomcp thinking add "Actually, approach B is better because..." -n 1 -t 3 --next --revision --revises 1
```

### Creating Branches

```bash
# Main sequence
biomcp thinking add "Main approach: use traditional method" -n 1 -t 5 --next
biomcp thinking add "Implement basic solution" -n 2 -t 5 --next

# Branch to explore alternative
biomcp thinking add "Alternative: try ML approach" -n 3 -t 5 --next --branch 2 --branch-id ml-approach

# View all branches
biomcp thinking branches
```

## Use Cases

1. **Problem Analysis**: Break down complex biomedical problems
2. **Research Planning**: Structure research approaches systematically
3. **Decision Making**: Document reasoning for treatment decisions
4. **Hypothesis Development**: Track evolving scientific hypotheses
5. **Protocol Design**: Plan clinical trial protocols step by step

## Tips

- Start with a clear problem statement as your first thought
- Use meaningful thought numbers that reflect logical progression
- Create branches when exploring alternative approaches
- Revise thoughts when new information changes your analysis
- Use the summary command to get an overview of your thinking process