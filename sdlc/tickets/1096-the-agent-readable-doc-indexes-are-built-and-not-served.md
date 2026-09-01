---
flow: build
priority: 7
---

# The agent-readable documentation indexes are built and are not served

Tickets 1073 and 1074 landed. `biomcp.org` still returns 404 for everything they produced. Checked 2026-09-01:

```
https://biomcp.org/                200
https://biomcp.org/llms.txt        404
https://biomcp.org/llms-full.txt   404
https://biomcp.org/docs/llms.txt   404
https://biomcp.org/concepts/what-is-biomcp.md   404
```

The site itself is live. The files exist in the repository and in the built output: `docs/llms.txt`, `docs/llms-full.txt`, `site/llms.txt`, `site/llms-full.txt`, `site/docs/llms.txt`. Record 1073 states that the indexes are published through the MkDocs site and that contract coverage keeps their links aligned so agents receive useful entry points instead of 404 responses. Against the live host, every one of those entry points is a 404 response.

So the coverage that gates the work passes against a local build, and the thing the work exists to do does not happen. Two tickets landed and the reader they were written for still cannot read anything.

This is the channel the original finding was about. `sdlc/issues/2026-08-28-biomcp-org-is-invisible-to-fetching-agents.md` recorded that agents which fetch rather than browse cannot read the site as text, at a time when chatgpt.com was already a top-six referrer to the repository.

## Required behavior

A check that claims a documentation index is published verifies it against the host that serves it, not against a directory on the machine that built it.

The agent-readable indexes and the markdown twins are reachable at the addresses the indexes themselves advertise.

A publish that does not carry these files to the live site is a visible failure rather than a silent one.

## Done, observably

- `curl https://biomcp.org/llms.txt` returns the file, not a 404.
- `llms-full.txt` and the markdown twin of a docs page are reachable on the live host.
- Every link inside the published `llms.txt` resolves on the live host.
- A future publish that drops these files fails a check rather than passing quietly.

## Boundary

This ticket does not rewrite the content of `llms.txt` or `llms-full.txt`, does not change which pages get markdown twins, and does not change the docs site's theme or navigation. Ticket 1075 covers the installed package carrying its own documentation, which is a separate delivery path and is not in scope. If the cause turns out to be deployment configuration rather than repository content, say so plainly in the record: a correct diagnosis that names an out-of-repo cause is a complete answer to this ticket.
