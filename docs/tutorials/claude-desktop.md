# BioMCP with Claude Desktop: Step-by-Step Tutorial

This tutorial will guide you through setting up BioMCP as a Model Context
Protocol (MCP) server for Claude Desktop, allowing Claude to access specialized
biomedical data.

## Prerequisites

- Claude Desktop: [Download from Anthropic](https://claude.ai/desktop)
- Python: 3.11 or newer
- uv: [Installation](https://docs.astral.sh/uv/getting-started/installation/)

Verify the uv installation on the command line:

```bash
uv --version
```

Make sure it is installed globally for Claude to access. For instance,
on MacOS, we recommend installing `uv` using Homebrew:

```bash
% which uv
/opt/homebrew/bin/uv
```

## Configure Claude Desktop

Open Claude Desktop and access the Settings > Developer section.

Then click "Edit Config" which on MacOS opens up the folder containing this
file:

```markdown
claude_desktop_config.json
```

Edit the file like this using your favorite text editor:

```json
{
  "biomcp": {
    "command": "uv",
    "args": ["run", "--with", "biomcp-python", "biomcp", "run"]
  }
}
```

We also recommend using Sequential Thinking MCP, which can be added as well
by making your whole file:

```json
{
  "mcpServers": {
    "biomcp": {
      "command": "uv",
      "args": ["run", "--with", "biomcp-python", "biomcp", "run"]
    },
    "sequential-thinking": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-sequential-thinking"]
    }
  }
}
```

For more information about Sequential Thinking:
[https://github.com/modelcontextprotocol/servers/tree/main/src/sequentialthinking](https://github.com/modelcontextprotocol/servers/tree/main/src/sequentialthinking)

Save your JSON file and restart Claude Desktop.

## Using BioMCP with Claude

Upload loading, we recommend accessing the BioMCP "custom instructions" by
clicking the "connector" icon below the chat that says "Attach from MCP".

In that dialog, select the biomcp instructions as an "integration" and the
MCP resource will be added as text.

You should see at least 9 tools that can be accessed with BioMCP plus any
other tools you have configured (10 total with sequential thinking, for
instance).

### Tool Parameters

All BioMCP tools include a required `call_benefit` parameter that helps the model explain why it's using a particular tool. When using any BioMCP tool, you must provide:

- `call_benefit`: A brief description of why the tool is being called and what benefit is expected from using it

This parameter helps improve tool usage transparency and allows for better reasoning about tool selection.

### Example Queries

Below are some example questions to try based on your use cases/research.

### Clinical Trials Queries

Try questions like:

- "Find Phase 3 clinical trials for lung cancer with immunotherapy"
- "Are there any recruiting breast cancer trials near Boston?"
- "What are the eligibility criteria for trial NCT04280705?"

### PubMed Articles Queries

Try questions like:

- "Summarize recent research on EGFR mutations in lung cancer"
- "Find articles about the relationship between BRAF mutations and melanoma"
- "Get the abstract of PubMed article 21717063"

### Genetic Variants Queries

Try questions like:

- "What's the clinical significance of the BRAF V600E mutation?"
- "Find pathogenic variants in the TP53 gene"
- "Explain the difference between Class I and Class III BRAF mutations"

### Combination Queries

Claude can combine multiple BioMCP tools in a single query:

- "I'm researching KRAS G12C mutations in lung cancer. Can you find:"
  1. The key characteristics of this mutation
  2. Recent clinical trials targeting it
  3. Significant research papers from the last 2 years

Claude can help with complex biomedical research workflows:

- I'm studying treatment resistance in ALK-positive lung cancer. Help me:
  1. Identify the main ALK fusion variants
  2. Find current clinical trials testing next-generation ALK inhibitors
  3. Summarize recent literature on resistance mechanisms

## Troubleshooting

### Common Issues

- "SPAWN ENOENT" Error:
  - Make sure `uv` is in your PATH
  - Try using the full path to `uv` in the configuration
- Claude doesn't use BioMCP
  - Verify you've correctly configured the MCP server
  - Check if your query is specific enough to trigger BioMCP usage
  - Ask Claude directly to search trials, variants, or articles using BioMCP
- No results returned
  - Your query may be too specific or use terms not in the databases
  - Try reformulating with more standard medical terminology

## Resources

- [BioMCP Documentation](https://biomcp.org)
- [Claude Desktop Documentation](https://docs.anthropic.com/claude/docs/claude-desktop)
- [Model Context Protocol (MCP) Guide](https://docs.anthropic.com/claude/docs/model-context-protocol)
