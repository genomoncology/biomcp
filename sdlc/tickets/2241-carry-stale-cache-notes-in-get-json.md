---
priority: 2
---

# 2241 — Carry stale-cache notes in get JSON

Status: OPEN.

Milestone: 0.9.4

## Outcome

When a single-record `get ... --json` answer is served from a stale cache, the JSON says so in an optional `notes` field, as search, article and GWAS JSON already do.

## Evidence

`sdlc/issues/2026-09-27-get-json-bodies-have-no-notes-channel-for-the-stale-cache-age.md`. PM ruling 2026-10-19: add the field as optional and absent when empty, so existing consumers are unaffected; reuse the wording and shape of the existing search notes.

## Keeps

Fresh answers produce byte-identical JSON. Markdown output is unchanged.

## Proof

A replay test serves a stale reply for one get command and finds the note in JSON; a fresh reply has no `notes` key. Update the JSON schema docs and the changelog. Mark the issue resolved.
