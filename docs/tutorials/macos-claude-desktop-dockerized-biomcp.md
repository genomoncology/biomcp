# Containerized BioMCP with Claude Desktop for Mac: Step-by-Step Tutorial

This tutorial will guide you through setting up BioMCP as a Model Context
Protocol (MCP) server for Claude Desktop for Mac, while having biomcp fully dockerized.

## Prerequisites

- Claude Desktop: [Download from Anthropic](https://claude.ai/desktop), or if you have [brew](https://brew.sh/), simply `brew install --cask claude`
- Docker: [Download from Docker.com](https://docs.docker.com/desktop/setup/install/mac-install/) or, again if you have [brew](https://brew.sh/), simply `brew install --cask docker`

## Steps

Create a file called `Dockerfile` in a directory of your choice with this content
```
FROM python:3.13-slim AS production
RUN apt-get update && rm -rf /var/lib/apt/lists/*
WORKDIR /app
RUN pip install biomcp-python
EXPOSE 8000
CMD ["biomcp","run"]
```

then in the same directory run the command
`docker build -t biomcp .`

Finally, enter the directory of Claude Desktop configs
`cd "$HOME/Library/Application Support/Claude/"`

and edit the file `claude_desktop_config.json` adding the biomcp server

```
{
  "mcpServers": {
    "biomcp": {
      "command": "docker",
      "args": [
        "run",
        "-i",
        "--rm",
        "biomcp:latest"],
      "env": {}
    }
  }
}
```
That's it; you should now have biomcp, fully containerized, available in your Macos Claude Desktop Application
