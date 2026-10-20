---
base: 6065a037b8934ce19d04ad769dd62b95b272c15e
head: a703c2cb778919aaaa20b4c288c610f25be0a0a2
---

# Let the CLI show the MCP tool catalog

`biomcp mcp tools` now prints the shared MCP catalog as a JSON array without
starting a server. The budget measurement wrapper calls that command, so it
measures the same installed-binary interface users can inspect and diff.
