# Claude Desktop Integration

This guide covers how to integrate BioMCP with Claude Desktop, enabling AI-powered biomedical research directly in your Claude conversations.

## Prerequisites

- [Claude Desktop](https://claude.ai/download) application
- One of the following:
  - **Option A**: Python 3.10+ and [uv](https://docs.astral.sh/uv/) (recommended)
  - **Option B**: [Docker](https://www.docker.com/products/docker-desktop/)

## Installation Methods

### Option A: Using uv (Recommended)

This method is fastest and easiest for most users.

#### 1. Install uv

```bash
# macOS/Linux
curl -LsSf https://astral.sh/uv/install.sh | sh

# Windows
powershell -c "irm https://astral.sh/uv/install.ps1 | iex"
```

#### 2. Configure Claude Desktop

Add BioMCP to your Claude Desktop configuration file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "biomcp": {
      "command": "uv",
      "args": ["run", "--with", "biomcp-python", "biomcp", "run"],
      "env": {
        "NCI_API_KEY": "your-nci-api-key-here",
        "ALPHAGENOME_API_KEY": "your-alphagenome-key-here",
        "CBIO_TOKEN": "your-cbioportal-token-here"
      }
    }
  }
}
```

### Option B: Using Docker

This method provides better isolation and consistency across systems.

#### 1. Create a Dockerfile

Create a file named `Dockerfile`:

```dockerfile
FROM python:3.11-slim

# Install BioMCP
RUN pip install biomcp-python

# Set the entrypoint
ENTRYPOINT ["biomcp", "run"]
```

#### 2. Build the Docker Image

```bash
docker build -t biomcp:latest .
```

#### 3. Configure Claude Desktop

Add BioMCP to your configuration file:

```json
{
  "mcpServers": {
    "biomcp": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "biomcp:latest"],
      "env": {
        "NCI_API_KEY": "your-nci-api-key-here",
        "ALPHAGENOME_API_KEY": "your-alphagenome-key-here",
        "CBIO_TOKEN": "your-cbioportal-token-here"
      }
    }
  }
}
```

## Verification

1. Restart Claude Desktop after updating the configuration
2. Start a new conversation
3. Look for the 🔌 icon indicating MCP is connected
4. Test with: "Can you search for articles about BRAF mutations in melanoma?"

## Setting Up API Keys

While BioMCP works without API keys, some features require them for full functionality:

### NCI API Key (Optional)

Enables access to NCI's clinical trials database with advanced filters:

- Get your key from [NCI API Portal](https://api.cancer.gov)
- Add to configuration as `NCI_API_KEY`

### AlphaGenome API Key (Optional)

Enables variant effect predictions using Google DeepMind's AlphaGenome:

- Register at [AlphaGenome Portal](https://alphagenome.google.com)
- Add to configuration as `ALPHAGENOME_API_KEY`

### cBioPortal Token (Optional)

Enables enhanced cancer genomics queries:

- Get token from [cBioPortal](https://www.cbioportal.org/webAPI)
- Add to configuration as `CBIO_TOKEN`

## Usage Examples

Once configured, you can ask Claude to perform various biomedical research tasks:

### Literature Search

```
"Find recent articles about CAR-T therapy for B-cell lymphomas"
```

### Clinical Trials

```
"Search for actively recruiting trials for EGFR-mutant lung cancer"
```

### Variant Analysis

```
"What is known about the pathogenicity of BRCA1 c.5266dupC?"
```

### Drug Information

```
"Tell me about the mechanism of action and indications for pembrolizumab"
```

### Complex Research

```
"I need a comprehensive analysis of treatment options for a patient with
BRAF V600E melanoma who has progressed on dabrafenib/trametinib"
```

## The Deep Researcher Persona

BioMCP includes a specialized "Deep Researcher" persona that enhances Claude's biomedical research capabilities:

- **Sequential Thinking**: Automatically uses the `think` tool for systematic analysis
- **Comprehensive Coverage**: Searches multiple databases and synthesizes findings
- **Evidence-Based**: Provides citations and links to primary sources
- **Clinical Focus**: Understands medical context and terminology

To activate, simply ask biomedical questions naturally. The persona automatically engages for research tasks.

## Troubleshooting

### "MCP Connection Failed"

1. Verify the configuration file path is correct
2. Check JSON syntax (no trailing commas)
3. Ensure Claude Desktop has been restarted
4. Check that uv or Docker is properly installed

### "Command Not Found"

**For uv**:

```bash
# Verify uv installation
uv --version

# Ensure PATH includes uv
echo $PATH | grep -q "\.local/bin" || echo "PATH needs updating"
```

**For Docker**:

```bash
# Verify Docker is running
docker ps

# Test BioMCP container
docker run -it --rm biomcp:latest --help
```

### "No Results Found"

- Check your internet connection
- Verify API keys are correctly set (if using optional features)
- Try simpler queries first
- Use official gene symbols (e.g., "TP53" not "p53")

### Performance Issues

**For uv**:

- First run may be slow due to package downloads
- Subsequent runs use cached environments

**For Docker**:

- Ensure Docker has sufficient memory allocated
- Consider building with `--platform` flag for Apple Silicon

## Advanced Configuration

### Custom Environment Variables

Add any additional environment variables your research requires:

```json
{
  "mcpServers": {
    "biomcp": {
      "command": "uv",
      "args": ["run", "--with", "biomcp-python", "biomcp", "run"],
      "env": {
        "BIOMCP_LOG_LEVEL": "DEBUG",
        "BIOMCP_CACHE_DIR": "/path/to/cache",
        "HTTP_PROXY": "http://your-proxy:8080"
      }
    }
  }
}
```

### Multiple Configurations

You can run multiple BioMCP instances with different settings:

```json
{
  "mcpServers": {
    "biomcp-prod": {
      "command": "uv",
      "args": ["run", "--with", "biomcp-python", "biomcp", "run"],
      "env": {
        "BIOMCP_ENV": "production"
      }
    },
    "biomcp-dev": {
      "command": "uv",
      "args": ["run", "--with", "biomcp-python@latest", "biomcp", "run"],
      "env": {
        "BIOMCP_ENV": "development",
        "BIOMCP_LOG_LEVEL": "DEBUG"
      }
    }
  }
}
```

## Best Practices

1. **Start Simple**: Test with basic queries before complex research tasks
2. **Be Specific**: Use official gene symbols and disease names
3. **Iterate**: Refine queries based on initial results
4. **Verify Sources**: Always check the provided citations
5. **Save Important Findings**: Export conversation or copy key results

## Getting Help

- **Documentation**: [BioMCP Docs](https://github.com/genomoncology/biomcp)
- **Issues**: [GitHub Issues](https://github.com/genomoncology/biomcp/issues)
- **Community**: [Discussions](https://github.com/genomoncology/biomcp/discussions)

## Next Steps

Now that BioMCP is integrated with Claude Desktop:

1. Try the [example queries](#usage-examples) above
2. Explore [How-to Guides](../how-to-guides/01-find-articles-and-cbioportal-data.md) for specific research workflows
3. Learn about [Sequential Thinking](../concepts/03-sequential-thinking-with-the-think-tool.md) for complex analyses
4. Set up [additional API keys](03-authentication-and-api-keys.md) for enhanced features
