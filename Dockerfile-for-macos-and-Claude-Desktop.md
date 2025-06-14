# How to containerize Biomcp and make it available on Claude Desktop for Mac

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