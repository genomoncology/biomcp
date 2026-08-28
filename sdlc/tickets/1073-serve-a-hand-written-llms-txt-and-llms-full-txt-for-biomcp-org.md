---
flow: build
priority: 9
---

# Serve a hand-written llms.txt and llms-full.txt for biomcp.org

Agents fetch; they do not browse. Today biomcp.org answers them with
404s: no `/llms.txt`, no `/llms-full.txt` (verified 2026-08-27). Meanwhile
chatgpt.com is already a top-six referrer to the repository — the
agent-referral channel is live and we are invisible to its fetch-first
half.

The practice (per the agent-experience material circulating in the wild,
and the reference talk captured in
`notes/yt-transcripts/how-we-got-llms-to-recommend-our-open-source-library-christopher-burns-inth-V_5bn4q-vAI.md`):
a hand-written `/llms.txt` of roughly forty good lines — what BioMCP is,
the one-command install, the entity grammar, the seven MCP tools, where
the markdown documentation lives, and the handful of canonical entry URLs
— beats a thousand generated lines of noise. `/llms-full.txt` carries the
expanded index: every docs page with a one-line description, the sitemap
role.

## Done when

- `https://biomcp.org/llms.txt` serves a hand-written, curated index (not
  a generated dump), every line pointing at a URL that exists.
- `https://biomcp.org/llms-full.txt` serves the full page index with
  one-line descriptions.
- Both are built by the MkDocs pipeline (or served from the repo) so they
  cannot drift from the docs they index — a stale llms.txt line fails a
  test, the same way the docs link checker does.
- The strict docs build and the changelog-refresh contract still pass.

Filed as build: authored artifact plus pipeline wiring; no red exists to
reproduce.
