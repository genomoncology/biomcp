---
priority: 2
---

# 2241 — Carry stale-cache notes in get JSON

Status: OPEN.

Milestone: 0.9.4

## Outcome

- Changes: `src/render/json.rs` `src/sources/mod.rs` `src/cli/stale_json_note_tests.rs` `skills/schemas/*.json` `CHANGELOG.md` `sdlc/issues/2026-09-27-get-json-bodies-have-no-notes-channel-for-the-stale-cache-age.md` and this ticket.

When a single-record `get ... --json` answer is served from a stale cache, the JSON says so in an optional `notes` field, as search, article and GWAS JSON already do.

## Evidence

`sdlc/issues/2026-09-27-get-json-bodies-have-no-notes-channel-for-the-stale-cache-age.md`. PM ruling 2026-10-19: add the field as optional and absent when empty, so existing consumers are unaffected; reuse the wording and shape of the existing search notes.

## Keeps

Fresh answers produce byte-identical JSON. Markdown output is unchanged.

## Proof

A replay test serves a stale reply for one get command and finds the note in JSON; a fresh reply has no `notes` key. Update the JSON schema docs and the changelog. Mark the issue resolved.

## Coordinator development brief

Add optional _meta.notes using the existing stale sentence wording, omitted when empty. Preserve fresh byte identity and Markdown output. Use a non-draining read if the common metadata builder also serves Markdown so it does not steal later notes. Schema definitions document the same optional field in the affected seven entity schemas. Avoid unrelated configuration/source docs already owned by2240. The isolated get-gene regression compares fresh output and then an expired cache with a stopped provider; require stale notes and unchanged other fields.

All builds and tests run through the configured build job. Stage private test/source patches on a published base without pushing WIP; retain patch and per-file hashes with raw evidence, then verify the final published source matches. Push source once ready for review. The developer may resolve the named issue findings as required by the outcome, but does not edit ticket status or send mail.
