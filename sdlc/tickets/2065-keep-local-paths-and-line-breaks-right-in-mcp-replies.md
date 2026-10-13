# 2065 — Keep local paths and line breaks right in MCP replies

Status: OPEN.

Milestone: 0.9.3

## Outcome

No MCP reply names a path on the server's machine, and MCP refusals keep the line breaks the CLI shows.

## Evidence

Filed 2026-10-13 from the post-release install and product test of v0.9.2 (tag `4049666f5`), run on Linux x86_64 and macOS arm64 against every install channel.

- The MCP `biomcp` tool with `{"command":"health"}` returns rows such as "EMA local data (/home/…/.local/share/biomcp/ema)" and "Cache dir (/tmp/…/cache)", and "/Users/…" on macOS. 0.9.2 refuses cache commands over MCP and removed the WHO data path from MCP replies for this reason. 0.9.1 does the same.
- MCP `get` for disease `MM` and for variant `TP53 S183Y` returns refusal text with no newlines ("…Candidates: - Miyoshi muscular dystrophy (MONDO:0009685) Clinical reading: …"); the CLI prints the same refusals on several lines. The 0.9.2 changelog says the numbering refusals' line breaks survive.

- Starts from: the public-error form 2044 added for MCP tool errors, and `cli/health`.
- Keeps: the CLI's health output with its paths.
- Changes: MCP health rows name the local dataset or cache without its path; the MCP error form keeps newlines.
- Proof: MCP tests through the server: health returns no string that starts with the test's temporary home or cache folder; the MM and S183Y refusals contain the same line breaks as the CLI's.
- Defers: nothing.
