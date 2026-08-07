---
base: b23d18b6246d37f6b64a6317805f3750919177b7
head: bd1d508b765f35ce237da907ece9673cfdb9d50c
---
`biomcp-cli` is fully installable (PyPI, install.sh, Claude Code plugin, Codex, Claude Desktop MCPB) but is NOT listed in the official MCP Registry (registry.modelcontextprotocol.io). That registry is free metadata — no hosting, no ongoing cost — and aggregators (Glama, PulseMCP, mcp.so) scrape it roughly hourly, so one listing cascades into the major MCP directories automatically. Separately, `pip install biomcp` installs an unrelated third-party package; users must install `biomcp-cli`. Both are fixed here: publish to the registry and disambiguate the package name across the install-facing surfaces.

Imported from March ticket 473. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/473-publish-biomcp-cli-to-the-official-mcp-registry-and-disambiguate-the-biomcp-pypi-name
