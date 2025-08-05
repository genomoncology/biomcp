# Authentication and API Keys

BioMCP integrates with multiple biomedical databases. While many features work without authentication, some advanced capabilities require API keys for enhanced functionality.

## Overview of API Keys

| Service         | Required?  | Features Enabled                                  | Get Key                                                                |
| --------------- | ---------- | ------------------------------------------------- | ---------------------------------------------------------------------- |
| **NCI API**     | Optional   | Advanced clinical trial filters, biomarker search | [api.cancer.gov](https://api.cancer.gov)                               |
| **AlphaGenome** | Required\* | Variant effect predictions                        | [deepmind.google.com](https://deepmind.google.com/science/alphagenome) |
| **cBioPortal**  | Optional   | Enhanced cancer genomics queries                  | [cbioportal.org](https://www.cbioportal.org/webAPI)                    |

\*Required only when using AlphaGenome features

## Setting Up API Keys

### Method 1: Environment Variables (Recommended for Personal Use)

Set environment variables in your shell configuration:

```bash
# Add to ~/.bashrc, ~/.zshrc, or equivalent
export NCI_API_KEY="your-nci-api-key"
export ALPHAGENOME_API_KEY="your-alphagenome-key"
export CBIO_TOKEN="your-cbioportal-token"
```

### Method 2: Configuration Files

#### For Claude Desktop

Add keys to your Claude Desktop configuration:

```json
{
  "mcpServers": {
    "biomcp": {
      "command": "uv",
      "args": ["run", "--with", "biomcp-python", "biomcp", "run"],
      "env": {
        "NCI_API_KEY": "your-nci-api-key",
        "ALPHAGENOME_API_KEY": "your-alphagenome-key",
        "CBIO_TOKEN": "your-cbioportal-token"
      }
    }
  }
}
```

#### For Docker Deployments

Include in your Docker run command:

```bash
docker run -e NCI_API_KEY="your-key" \
           -e ALPHAGENOME_API_KEY="your-key" \
           -e CBIO_TOKEN="your-token" \
           biomcp:latest
```

### Method 3: Per-Request Keys (For Hosted Environments)

When using BioMCP through AI assistants or hosted services, provide keys in your request:

```
"Predict effects of BRAF V600E mutation. My AlphaGenome API key is YOUR_KEY_HERE"
```

The AI will recognize patterns like "My [service] API key is..." and use the key for that request only.

## Individual Service Setup

### NCI Clinical Trials API

The National Cancer Institute API provides advanced clinical trial search capabilities.

#### Getting Your Key

1. Visit [api.cancer.gov](https://api.cancer.gov)
2. Click "Get API Key"
3. Complete registration
4. Key is emailed immediately

#### Features Enabled

- Advanced biomarker-based trial search
- Organization and investigator lookups
- Intervention and disease vocabularies
- Higher rate limits (1000 requests/day vs 100)

#### Usage Example

```bash
# With API key set
export NCI_API_KEY="your-key"

# Search trials with biomarker criteria
biomcp trial search --condition melanoma --source nci \
  --required-mutations "BRAF V600E" --allow-brain-mets true
```

### AlphaGenome

Google DeepMind's AlphaGenome predicts variant effects on gene expression and chromatin accessibility.

#### Getting Your Key

1. Visit [AlphaGenome Portal](https://deepmind.google.com/science/alphagenome)
2. Register for non-commercial use
3. Receive API key via email
4. Accept terms of service

#### Features Enabled

- Gene expression predictions
- Chromatin accessibility analysis
- Splicing effect predictions
- Tissue-specific analyses

#### Usage Examples

**CLI with environment variable:**

```bash
export ALPHAGENOME_API_KEY="your-key"
biomcp variant predict chr7 140753336 A T
```

**CLI with per-request key:**

```bash
biomcp variant predict chr7 140753336 A T --api-key YOUR_KEY
```

**Through AI assistant:**

```
"Predict regulatory effects of BRAF V600E (chr7:140753336 A>T).
My AlphaGenome API key is YOUR_KEY_HERE"
```

### cBioPortal

The cBioPortal token enables enhanced cancer genomics queries.

#### Getting Your Token

1. Create account at [cbioportal.org](https://www.cbioportal.org)
2. Navigate to "Web API" section
3. Generate a personal access token
4. Copy the token (shown only once)

#### Features Enabled

- Higher API rate limits
- Access to private studies (if authorized)
- Batch query capabilities
- Extended timeout limits

#### Usage

cBioPortal integration is automatic when searching for genes. The token enables:

```bash
# Enhanced gene search with cancer genomics
export CBIO_TOKEN="your-token"
biomcp article search --gene BRAF --disease melanoma
```

## Security Best Practices

### DO:

- Store keys in environment variables or secure config files
- Use per-request keys in shared/hosted environments
- Rotate keys periodically
- Use separate keys for development/production

### DON'T:

- Commit keys to version control
- Share keys with others
- Include keys in code or documentation
- Store keys in plain text files

### Git Security

Add to `.gitignore`:

```
.env
.env.local
*.key
config/secrets/
```

Use git-secrets to prevent accidental commits:

```bash
# Install git-secrets
brew install git-secrets  # macOS
# or follow instructions at github.com/awslabs/git-secrets

# Set up in your repo
git secrets --install
git secrets --register-aws  # Detects common key patterns
```

## Troubleshooting

### "API Key Required" Errors

**For AlphaGenome:**

- This service always requires a key
- Provide it via environment variable or per-request
- Check key spelling and format

**For NCI:**

- Basic search works without key
- Advanced features require authentication
- Verify key is active at api.cancer.gov

### "Invalid API Key" Errors

1. Check for extra spaces or quotes
2. Ensure key hasn't expired
3. Verify you're using the correct service's key
4. Test key directly with the service's API

### Rate Limit Errors

**Without API keys:**

- Public limits are restrictive (e.g., 100 requests/day)
- Add delays between requests
- Consider getting API keys

**With API keys:**

- Limits are much higher but still exist
- Implement exponential backoff
- Cache results when possible

## Testing Your Setup

### Check Environment Variables

```bash
# List all BioMCP-related environment variables
env | grep -E "(NCI_API_KEY|ALPHAGENOME_API_KEY|CBIO_TOKEN)"
```

### Test Each Service

```bash
# Test NCI API
biomcp trial search --condition cancer --source nci --limit 1

# Test AlphaGenome (requires key)
biomcp variant predict chr7 140753336 A T --limit 1

# Test cBioPortal integration
biomcp article search --gene TP53 --limit 1
```

## API Key Management Tools

For managing multiple API keys securely:

### 1. direnv (Recommended)

```bash
# Install direnv
brew install direnv  # macOS
# Add to shell: eval "$(direnv hook zsh)"

# Create .envrc in project
echo 'export NCI_API_KEY="your-key"' > .envrc
direnv allow
```

### 2. 1Password CLI

```bash
# Store in 1Password
op item create --category=password \
  --title="BioMCP API Keys" \
  --vault="Development" \
  NCI_API_KEY="your-key"

# Load in shell
export NCI_API_KEY=$(op read "op://Development/BioMCP API Keys/NCI_API_KEY")
```

### 3. AWS Secrets Manager

```bash
# Store secret
aws secretsmanager create-secret \
  --name biomcp/api-keys \
  --secret-string '{"NCI_API_KEY":"your-key"}'

# Retrieve in script
export NCI_API_KEY=$(aws secretsmanager get-secret-value \
  --secret-id biomcp/api-keys \
  --query SecretString \
  --output text | jq -r .NCI_API_KEY)
```

## Next Steps

Now that you have API keys configured:

1. Test each service to ensure keys work
2. Explore [How-to Guides](../how-to-guides/01-find-articles-and-cbioportal-data.md) for advanced features
3. Set up [logging and monitoring](../how-to-guides/05-logging-and-monitoring-with-bigquery.md)
4. Review [security policies](../policies.md) for your organization
