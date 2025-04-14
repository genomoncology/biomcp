# BioMCP Python SDK Tutorial

## Overview

The BioMCP Python SDK allows you to integrate BioMCP's biomedical data access capabilities directly into your Python applications. This tutorial provides a high-level overview of the SDK's capabilities.

## Key Features

- **Asynchronous API**: All BioMCP functions are async, designed for efficient network operations
- **JSON or Markdown Output**: Choose between formatted Markdown for display or JSON for programmatic use
- **Validated Models**: Input using Pydantic models for type safety and validation
- **Complete API Coverage**: Access all BioMCP capabilities (variants, trials, articles) programmatically

## Main Components

The SDK is organized into domain-specific modules:

1. **Variants Module**

   - Search for genetic variants with `search_variants()`
   - Retrieve detailed variant information with `get_variant()`

2. **Trials Module**

   - Search for clinical trials with `search_trials()`
   - Get trial details with `get_trial()`

3. **Articles Module**
   - Search for medical literature with `search_articles()`
   - Retrieve article details with `fetch_articles()`

## Basic Usage Pattern

All BioMCP SDK functions follow a similar pattern:

1. Import required modules and models
2. Create a query object (e.g., `VariantQuery`, `TrialQuery`)
3. Call the async function with the query
4. Process the results (as Markdown or JSON)

## Example Code

To use the below code, either use the `uv` script runner or install the
biomcp-python package using pip or uv.

```bash
pip install biomcp-python
```

or uv pip install:

```bash
uv pip install biomcp-python
```

or add the package to your uv project:

```bash
uv add biomcp-python
```

or run the script directly:

```python
#!/usr/bin/env -S uv --quiet run --script
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "biomcp-python",
# ]
# ///

import json

from biomcp.variants.search import VariantQuery, search_variants

async def find_pathogenic_tp53():
    # noinspection PyTypeChecker
    query = VariantQuery(gene="TP53", significance="pathogenic", size=5)
    # Get results as Markdown (default)
    json_output_str = await search_variants(query, output_json=True)
    data = json.loads(json_output_str)
    assert len(data) == 5
    for item in data:
        clinvar = item.get("clinvar")
        for rcv in clinvar.get("rcv", []):
            assert "pathogenic" in rcv["clinical_significance"].lower()

```

For complete examples of the BioMCP Python SDK in action, see the official example script:

[BioMCP Python SDK Example Script](https://github.com/genomoncology/biomcp/blob/main/example_scripts/python_sdk.py)

## Next Steps

For more detailed information on the SDK:

- Explore the source code at [github.com/genomoncology/biomcp](https://github.com/genomoncology/biomcp)
- Run `help()` on imported modules and classes for API details
