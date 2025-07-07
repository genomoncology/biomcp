# AlphaGenome API Key Handling

This guide explains how to use AlphaGenome predictions in BioMCP, especially in hosted environments where users need to provide their own API keys.

## Overview

AlphaGenome requires an API key for predictions. BioMCP supports three ways to provide it:

1. **Per-request API key** (recommended for hosted environments)
2. **Environment variable** (for personal use)
3. **Interactive prompt** (AI agents will ask for it)

## Per-Request API Key (Recommended for Hosted Environments)

When using BioMCP through a hosted service or AI assistant, include your API key directly in your request:

### Example 1: Natural Language

```
"Predict regulatory effects of BRAF V600E mutation: chr7:140753336 A>T.
My AlphaGenome API key is YOUR_KEY_HERE"
```

The AI agent will recognize the pattern "My AlphaGenome API key is..." and extract the key automatically.

### Example 2: Multiple Predictions

```
"I need to analyze several variants. My AlphaGenome API key is YOUR_KEY_HERE.
Please predict effects for:
1. BRAF V600E (chr7:140753336 A>T)
2. TP53 R273H (chr17:7577121 C>T)
3. EGFR L858R (chr7:55259515 T>G)"
```

## Environment Variable (Personal Use)

For personal use or development, set the API key as an environment variable:

```bash
export ALPHAGENOME_API_KEY='your-key-here'
```

## Interactive Prompt

If no API key is provided, the AI agent will receive a clear action request:

````
❌ **AlphaGenome API key required**

I need an API key to use AlphaGenome. Please provide it by either:

**Option 1: Include your key in your request**
Say: "My AlphaGenome API key is YOUR_KEY_HERE" and I'll use it for this prediction.

**Option 2: Set it as an environment variable (for persistent use)**
```bash
export ALPHAGENOME_API_KEY='your-key'
````

Get a free API key at: https://deepmind.google.com/science/alphagenome

**ACTION REQUIRED**: Please provide your API key using Option 1 above to continue.

````

## CLI Usage

The CLI also supports per-request API keys:

```bash
# Using environment variable
export ALPHAGENOME_API_KEY='your-key'
biomcp variant predict chr7 140753336 A T

# Using command-line option
biomcp variant predict chr7 140753336 A T --api-key YOUR_KEY

# The --api-key option overrides the environment variable
````

## Security Considerations

1. **Hosted Environments**: Always use per-request API keys to ensure users' keys aren't shared
2. **Never commit API keys** to version control
3. **Use environment variables** only on trusted personal machines
4. **API keys are user-specific** - each user should obtain their own

## Getting an API Key

1. Visit https://deepmind.google.com/science/alphagenome
2. Register for a free API key (non-commercial use)
3. Keep your key secure and don't share it

## Troubleshooting

### AI Agent Doesn't Ask for Key

If the AI agent doesn't proactively ask for your API key, include it in your initial request:

- ✅ "Predict effects of chr7:140753336 A>T. My AlphaGenome API key is KEY123"
- ❌ "Predict effects of chr7:140753336 A>T" (agent may continue without asking)

### Key Not Recognized

Ensure you use the exact phrase "My AlphaGenome API key is" followed by your key.

### Invalid Key Error

Verify your key is correct and active at the AlphaGenome website.

## Best Practices

1. **For AI Assistants**: Always include your API key in the initial request
2. **For Scripts**: Use environment variables for automation
3. **For Hosted Services**: Never store users' API keys; always use per-request keys
4. **For Development**: Use a separate development API key

## Example Conversation

**User**: "Can you predict the effects of the BRAF V600E mutation?"

**AI**: "I'll help you predict the effects of the BRAF V600E mutation using AlphaGenome. I'll need your AlphaGenome API key to proceed. Please provide it by saying 'My AlphaGenome API key is YOUR_KEY_HERE'."

**User**: "My AlphaGenome API key is AIzaSyB..."

**AI**: "Thank you! Now analyzing the BRAF V600E mutation (chr7:140753336 A>T)..."

[Results follow]
