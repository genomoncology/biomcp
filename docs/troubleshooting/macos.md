# MacOS Troubleshooting Guide

**Prerequisites:**

- Ensure you have `uv` installed. Recommended method for macOS:
  ```bash
  brew install uv
  ```
  For other systems or methods, see the [uv installation guide](https://docs.astral.sh/uv/install/).
- Ensure you have `npx` available (usually comes with Node.js/npm). Recommended method for macOS if needed:
  ```bash
  brew install node
  ```

**1. Testing the CLI Directly:**

You can run `biomcp` commands directly without a full installation using `uv`:

- Check the version:
  ```bash
  uv run --with biomcp-python biomcp version
  # Expected Output (version may vary): biomcp version: 0.1.0
  ```
- Test a search command (e.g., trial search):
  ```bash
  uv run --with biomcp-python biomcp trial search --condition NSCLC | head -n 5
  # Expected Output (NCT ID and Title will vary):
  # # Record 1
  # Nct Number: NCT0XXXXXXX
  # Study Title:
  #   Some Title Related to NSCLC
  # Study Url: https://clinicaltrials.gov/study/NCT0XXXXXXX
  ```

**2. Testing the MCP Server with Inspector:**

This verifies that the server starts correctly and the tools are available via the Model Context Protocol.

- Run the inspector, telling it to start your server using the `uv` command:
  ```bash
  npx @modelcontextprotocol/inspector uv run --with biomcp-python biomcp run
  ```
- Open the MCP Inspector interface in your browser (usually `http://127.0.0.1:6274`).
- You should see the list of available tools (e.g., `article_searcher`, `trial_protocol`, `variant_searcher`, etc.).
- Try invoking a tool:
  - Select `trial_searcher`.
  - Enter valid JSON input matching the `TrialQuery` model, for example:
    ```json
    {
      "conditions": ["Melanoma"],
      "recruiting_status": "OPEN"
    }
    ```
  - Click "Call Tool".
  - You should see a Markdown-formatted list of results in the "Output" section.
