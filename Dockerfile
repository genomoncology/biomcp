# Generated by https://smithery.ai. See: https://smithery.ai/docs/config#dockerfile
FROM python:3.11-slim

# set work directory
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends gcc build-essential && rm -rf /var/lib/apt/lists/*

# Copy requirements (pyproject.toml, etc.)
COPY pyproject.toml .
COPY README.md .
COPY LICENSE .

# Copy source code
COPY src ./src
COPY tests ./tests
COPY Makefile .
COPY tox.ini .

# Install the package
RUN pip install --upgrade pip && pip install .

# Expose port if necessary
# EXPOSE 8000

# Run the MCP server
CMD ["biomcp", "run"]
