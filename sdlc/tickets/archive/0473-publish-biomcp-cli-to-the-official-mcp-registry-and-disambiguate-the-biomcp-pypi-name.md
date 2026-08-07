---
flow: build
priority: 5
---
# Publish biomcp-cli to the official MCP Registry and disambiguate the biomcp PyPI name

`biomcp-cli` is fully installable (PyPI, install.sh, Claude Code plugin, Codex, Claude Desktop MCPB) but is NOT listed in the official MCP Registry (registry.modelcontextprotocol.io). That registry is free metadata — no hosting, no ongoing cost — and aggregators (Glama, PulseMCP, mcp.so) scrape it roughly hourly, so one listing cascades into the major MCP directories automatically. Separately, `pip install biomcp` installs an unrelated third-party package; users must install `biomcp-cli`. Both are fixed here: publish to the registry and disambiguate the package name across the install-facing surfaces.

Completed under March on 2026-07-01, as March ticket 473. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/473-publish-biomcp-cli-to-the-official-mcp-registry-and-disambiguate-the-biomcp-pypi-name
