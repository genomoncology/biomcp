---
priority: 2
---

# 2241 — Carry stale-cache notes in get JSON

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision 362bf8f, accept

Reviews: revision c513540, accept

Reviews: revision 2b025b6, accept

Reviews: revision 7b3cedb, accept

## Outcome

- Changes: `CHANGELOG.md` `sdlc/issues/2026-09-27-get-json-bodies-have-no-notes-channel-for-the-stale-cache-age.md` `sdlc/tickets/1260-unify-variant-transcript-numbering-in-lookup-and-display.md` `sdlc/tickets/2237-repair-garbled-gtr-trademark-characters.md` `sdlc/tickets/2238-reject-limit-and-offset-in-variant-erepo-caid-mode.md` `sdlc/tickets/2239-cap-the-drug-trial-alias-cache.md` `sdlc/tickets/2240-use-the-ca-bundle-for-alphagenome.md` `sdlc/tickets/2242-settle-three-suspected-bugs-with-live-runs.md` `sdlc/tickets/2243-fix-the-remaining-test-reliability-leftovers.md` `sdlc/tickets/2244-find-the-single-cpu-gencc-lease-hang.md` `skills/schemas/article.json` `skills/schemas/disease.json` `skills/schemas/drug.json` `skills/schemas/gene.json` `skills/schemas/pathway.json` `skills/schemas/trial.json` `skills/schemas/variant.json` `src/cli/outcome.rs` `src/cli/shared.rs` `src/cli/shared/get_notes.rs` `src/cli/stale_json_note_tests.rs` and this ticket. Carried PM ticket and milestone records are unchanged except this ticket own reviewed proof.

When a single-record `get ... --json` answer is served from a stale cache, the JSON says so in an optional `notes` field, as search, article and GWAS JSON already do.

## Evidence

`sdlc/issues/2026-09-27-get-json-bodies-have-no-notes-channel-for-the-stale-cache-age.md`. PM ruling 2026-10-19: add the field as optional and absent when empty, so existing consumers are unaffected; reuse the wording and shape of the existing search notes.

## Keeps

Fresh answers produce byte-identical JSON. Markdown output is unchanged.

## Proof

A replay test serves a stale reply for one get command and finds the note in JSON; a fresh reply has no `notes` key. Update the JSON schema docs and the changelog. Mark the issue resolved.

## Coordinator development brief

Attach optional _meta.notes at successful get-JSON finalization inside both existing CLI/MCP stale scopes, including author. Reuse the existing sentence collector. Preserve original bytes immediately when notes are empty; append and deduplicate existing notes rather than replacing them. Markdown remains unchanged. A small shared helper avoids per-entity adapters and changing the collector API. This scope does not collide with2238 or2240. The affected JSON schemas document the optional field; no unrelated source/configuration docs change.

Red job2241/1792420478415-45863 on239d7adc4 plus the test-only patch failed the expected stale-get note assertion: _meta contained evidence_urls and next_commands but no notes. Fresh absence and repeated-byte checks passed before the stale assertion. Preserve that get-gene boundary and add focused coverage for the separate author routing that motivated central finalization, without building a second fake protocol framework.

Private source/test patch staging on a published base through pm job is authorized; retain exact patch and per-file hashes and verify final published correspondence. All builds and tests run through the configured build job. Push source once ready for review. Resolve only the explicitly named issue finding; never edit ticket status or send mail.

## Focused proof

Sourcedcb09dedb03f78928c9ec7096097fff6ae9d7eb2 attaches optional notes in shared get-JSON finalization for both CLI and MCP, including author. Fresh bytes return unchanged before parsing; Markdown remains unchanged. Red1792420478415-45863 failed the missing stale-note assertion. Green1792420727884-87303 passed two outside-in regressions, formatting, all-target Clippy, quality ratchet and seven schema checks. All thirteen committed file hashes matched the staged checked source before publication. Durable build-host experiment784-biomcp-2241-get-cache-notes retains biomcp-2241-retained-proof.tar.gz, SHA2567f860789c42addfadd308f863e1cf356499a7a27ddc8f6fe346194ffa959a92a. Original private local experiment489 retains duplicate raw evidence; no new live-service or broad-suite run was needed.

Fresh review accepted 362bf8fed64224b214898bc228cb9a6b8d8c912d. All get routes use the shared finalizer inside both collector scopes. An extra author fixture failed before producing an outcome and therefore did not exercise this finalizer; it was removed, with the diagnosis retained. Final product bytes match the originally reviewed source and original CLI/MCP gene proof; formatting and quality checks also passed in 1792421678983-52393. No author cache-policy change is claimed.

Keep: experiments/489-biomcp-2241-get-cache-notes — Retain cited raw proof until the release evidence is explicitly released.
Keep: experiments/784-biomcp-2241-get-cache-notes — Retain cited raw proof until the release evidence is explicitly released.
