# biomcp.org is invisible to fetching agents: no llms.txt, no markdown twins

Checked 2026-08-28: `https://biomcp.org/llms.txt`, `/llms-full.txt`, and `.md` twins of docs pages all return 404. Agents that fetch rather than browse cannot read the site as text. This matters because the channel already works for us without any steering: chatgpt.com is a top-6 referrer to the GitHub repo, and August 2026 was the best star month since March.

Source: a talk by Christopher Burns (c15t, ~3M npm downloads) on LLM recommendations becoming his top inbound channel. His numbers are self-reported and he sells a framework — discount the numbers, keep the mechanics. The mechanics are independently sound and cheap.

Wanted, in leverage order:

1. **Hand-written `llms.txt` (~40 lines)** at the site root: what BioMCP is, the tools, the one-command install, where the markdown lives. Quality beats volume — 40 good lines over 1,000 generated ones. Plus `llms-full.txt` with the full docs text concatenated.
2. **Markdown twins of every docs page.** The site is MkDocs, so the source is already markdown; a plugin can serve `.md` beside each page, or `llms.txt` can point at the raw GitHub sources until a proper route lands.
3. **Docs inside the package.** Coding agents read the installed package and the repo, not the website. Ship the markdown docs in the published package with an index file; Burns measures roughly half the tokens saved versus scraping HTML (self-reported, but directionally obvious).

Acceptance: `curl https://biomcp.org/llms.txt` returns the hand-written file; each docs page has a fetchable markdown twin the llms.txt links to; the published package carries the docs directory and an index.

Not in scope: Web-MCP endpoints (interesting, separate decision).
