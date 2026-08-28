---
flow: build
priority: 8
---

# Serve a markdown twin for every biomcp.org page

Agents that do fetch pages pay for HTML they cannot use. The emerging
convention — `.md` twins: every docs page served as raw markdown at its
own URL plus `.md`, with an HTTP header announcing the alternative — is
nearly free for us because MkDocs's *source* is already markdown; only
the serving is missing (verified 2026-08-27:
`https://biomcp.org/concepts/what-is-biomcp.md` → 404).

## Done when

- Every published docs page is also served as markdown at `<page>.md`,
  straight from the same source MkDocs builds from, so the twin cannot
  drift from the page.
- Pages carry the alternative-content header announcing the markdown
  twin, the way the major docs sites now do.
- At least one additional access path works for agents that cannot
  append paths or headers: content negotiation on an agent-marked
  Accept header, or a `?mode=agent` query — the design picks one and
  documents it in `llms.txt`.
- The strict build, link checker, and docs contracts still pass; twins
  are excluded from any HTML-only checks that would double-count them.

Filed as build: pipeline work with a design choice (which access paths),
no red to reproduce.
