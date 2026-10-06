# 1303 — Accept ORCID's real visibility casing in get author

Status: OPEN.

## Outcome

`biomcp get author orcid:<id>` resolves with a valid token, because the
person read accepts the visibility value ORCID actually sends. GitHub
issue #289.

## Evidence

- Live probe 2026-10-06: `GET pub.orcid.org/v3.0/0000-0002-1825-0097/person`
  with `Accept: application/vnd.orcid+json` returns `"visibility": "public"`
  — lowercase — beside a well-formed name.
- `src/sources/orcid.rs` `public_display_name` requires `Some("PUBLIC")`
  exactly, so every person read fails the gate, becomes a contract Api
  error, and the error projection flattens it to the generic
  "API request to ORCID failed. Retry the remote source."
- `author papers` works because the works path never reads visibility.
- The inline fixtures in `src/sources/orcid/tests.rs` hand-wrote
  `"visibility":"PUBLIC"`, a shape ORCID never sends, so the tests pinned
  the bug.
- Issue #289's secondary notes (percent-encoding the scope in the token
  tutorial form; anonymous public reads) belong in
  `docs/getting-started/api-keys.md`.

## Change detail

1. Compare visibility case-insensitively (public/PUBLIC both pass) in
   `src/sources/orcid.rs`.
2. Correct the inline person fixtures to the recorded lowercase shape and
   add a recorded capture of the demo record's person response with a
   receipt.
3. Add a case table test: public, PUBLIC, limited, private, absent.

## Keeps

- The works path, credential handling, and every sanitized message stay
  unchanged.

## Proof

- `get author orcid:0000-0002-1825-0097` resolves against a recorded
  lowercase fixture; a visibility-limited record still refuses cleanly.
- `docs/getting-started/api-keys.md` documents the encoded scope and the
  anonymous-read note.

## Priority note

P1: this breaks every `get author orcid:` call for every user with a
valid token.
