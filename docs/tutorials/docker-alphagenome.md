# Using AlphaGenome with Docker

This guide explains how to use AlphaGenome with BioMCP in Docker containers.

## Docker Setup

The BioMCP Docker image now includes AlphaGenome pre-installed. Here's how to use it:

### 1. Build the Docker Image

```bash
docker build -t biomcp-alphagenome .
```

### 2. Run with Docker Compose

Create or update your `docker-compose.yml`:

```yaml
services:
  biomcp-server:
    build: .
    image: biomcp-alphagenome:latest
    container_name: biomcp-server
    ports:
      - "8000:8000"
    environment:
      - MCP_MODE=worker # Can be 'stdio' or 'worker'
      - ALPHAGENOME_API_KEY=${ALPHAGENOME_API_KEY}
    restart: unless-stopped
```

### 3. Set Your API Key

Create a `.env` file in the same directory as your `docker-compose.yml`:

```bash
ALPHAGENOME_API_KEY=your-api-key-here
```

Or set it in your shell:

```bash
export ALPHAGENOME_API_KEY='your-api-key-here'
```

### 4. Start the Container

```bash
docker-compose up -d
```

## Usage in Container

### Via Docker Exec

You can run AlphaGenome predictions directly in the container:

```bash
# Basic prediction
docker exec biomcp-server biomcp variant predict chr7 140753336 A T

# With tissue specificity
docker exec biomcp-server biomcp variant predict chr7 140753336 A T --tissue UBERON:0000310
```

### Via MCP Worker Mode

When running in worker mode, the container exposes port 8000 for MCP connections. Configure your MCP client to connect to:

```
http://localhost:8000
```

## Dockerfile Details

The Dockerfile includes these AlphaGenome-specific changes:

1. **Git Installation**: Required to clone AlphaGenome repository

   ```dockerfile
   RUN apt-get update && apt-get install -y git
   ```

2. **AlphaGenome Installation**: Cloned and installed during build

   ```dockerfile
   RUN git clone https://github.com/google-deepmind/alphagenome.git /tmp/alphagenome && \
       pip install /tmp/alphagenome && \
       rm -rf /tmp/alphagenome
   ```

3. **Environment Variable**: API key passed through docker-compose
   ```yaml
   environment:
     - ALPHAGENOME_API_KEY=${ALPHAGENOME_API_KEY}
   ```

## Troubleshooting

### Container Can't Find API Key

Check that the environment variable is set:

```bash
docker exec biomcp-server env | grep ALPHAGENOME
```

### AlphaGenome Import Errors

Verify AlphaGenome is installed in the container:

```bash
docker exec biomcp-server pip list | grep alphagenome
```

### Rebuilding After Changes

If you update the Dockerfile or dependencies:

```bash
docker-compose build --no-cache
docker-compose up -d
```

## Security Considerations

1. **API Key Security**: Never commit your API key to version control
2. **Use .env Files**: Keep API keys in `.env` files (add to `.gitignore`)
3. **Network Security**: In production, use proper network isolation
4. **Volume Mounts**: Be careful with volume mounts that might expose sensitive data

## Example Docker Run Command

For standalone container without docker-compose:

```bash
docker run -d \
  --name biomcp-alphagenome \
  -p 8000:8000 \
  -e MCP_MODE=worker \
  -e ALPHAGENOME_API_KEY="$ALPHAGENOME_API_KEY" \
  biomcp-alphagenome:latest
```

## Performance Notes

- AlphaGenome predictions require network calls to Google's API
- Container startup is slightly slower due to AlphaGenome installation
- Consider using volume mounts for caching if making many predictions
- The 1Mb genomic window option uses more memory and takes longer

## Next Steps

- See [AlphaGenome Setup Guide](alphagenome-setup.md) for general setup
- See [AlphaGenome Prompt Examples](alphagenome-prompts.md) for usage patterns
- Check the main [Dockerfile](../../Dockerfile) for implementation details
