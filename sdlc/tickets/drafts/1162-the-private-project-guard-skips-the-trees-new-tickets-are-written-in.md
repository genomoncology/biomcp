---
flow: build
priority: 8
---

# The private-project guard skips the trees new tickets are written in

## Goal

The mechanical check that keeps outside project names out of this repository covers the trees where new living content is written. Today tickets and drafts are unscanned, so in the one tree that grows every week the rule depends on somebody remembering it.

`tests/test_documentation_consistency_audit_contract.py::test_current_markdown_does_not_depend_on_private_project_context` landed with record 1123 on 2026-09-05. It reads `docs/`, `architecture/`, and `sdlc/issues/`, and matches three literal marker strings. `sdlc/tickets/` and `sdlc/tickets/drafts/` are not read.

Reconfirmed at commit `b2e05326`: no genuinely open ticket and no draft contains one of the three tracked markers, so this is a guard-coverage gap rather than a present violation. Record 1123's own current-facts section notes that a withdrawn ticket already carried this class in an active-ticket occurrence.

## The choice to settle

The guard matches three literal strings. Widening the roots closes the tree gap; it does not catch a fourth private name in a new file. Widening the roots belongs here. Turning the marker list into a pattern is a separate question with its own false-positive cost against biomedical prose, and record 1123 already ruled that the bare word for one project must stay legal. Leave that out.

The scan must follow ticket lifecycle state rather than treating the whole tickets tree as current prose. Scan every Markdown file directly under `sdlc/tickets/` whose parsed numeric ID has no record filename beginning with the same numeric ID, plus every Markdown file under `sdlc/tickets/drafts/`. Do not scan record-backed top-level tickets or `sdlc/tickets/archive/`: both are historical evidence protected by record 1123.

## Done, observably

- Open top-level tickets and drafts are scanned alongside the roots already covered.
- Temporary-tree regressions prove that an active top-level ticket and a draft carrying a tracked marker fail.
- A top-level ticket with a matching numeric-ID completion record passes even when the two slugs differ, as does an archived ticket carrying the same marker.
- `sdlc/records/` stays unscanned because record 1123 forbids rewriting append-only history.
- Public dependency and provider names, this project's own hosting and attribution slug, and defect evidence that does not contain a tracked private marker keep passing. A tracked marker in living prose fails even when it is presented as historical evidence; any future exception requires an explicit, separately reviewed allowlist.

## Boundary

This ticket makes the existing check lifecycle-aware and widens the current-content trees it reads. It does not turn the marker list into a pattern, rewrite records or archived tickets, ban biomedical prose that reuses a project word, or add a second checker.
