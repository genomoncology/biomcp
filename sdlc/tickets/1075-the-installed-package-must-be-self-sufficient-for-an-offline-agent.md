---
flow: build
priority: 8
---

# The installed package must be self-sufficient for an offline agent

Coding agents that use BioMCP rarely visit biomcp.org — they work from
the installed package and the repository, and from training data that may
be stale. BioMCP already owns the strongest form of this play
(`biomcp skill install` ships worked workflows into the agent's own
space), but it is not verified that the published wheel and archive
carry the full agent surface: the skills, the docs markdown, and an
index file that tells a freshly-installed agent where everything lives.

## Done when

- The built wheel (and each published archive) contains the skills
  markdown and the docs markdown, plus one index file — `AGENTS.md` or
  the equivalent the ecosystem settles on — that names: what the package
  is, how to run the CLI, where the skills live, where each docs topic
  lives, and the canonical URLs for the live site.
- A test asserts the packaged artifact's contents against that inventory,
  so a packaging regression that drops the skills or docs fails the
  build rather than shipping silently.
- Installing from the wheel and running `biomcp skill install` with no
  network succeeds, pinned by a test.

The success metric the reference material claims for this shape is real
token savings for every agent that reaches for the package; we measure
what we can — presence, completeness, offline success — and the rest
follows.

Amendment (2026-08-30, answering the code-review refusal): the
release evidence assertions stay. Tests must keep asserting the
inspector's `archive_members` evidence field, with the expected
count computed dynamically from the executable set plus the
packaged inventory this ticket adds — never a deleted assertion
and never a hardcoded count that the new content invalidates.
Independent exact-member checks are welcome additions, not
replacements. Do not remove or weaken any other shipped evidence
field's assertion.
