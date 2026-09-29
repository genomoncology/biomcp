# Get-JSON bodies have no notes channel for the stale-cache age

Filed from ticket 1256 (scope corrected after the review's P0-3
fix). The stale-cache age reaches three of the four output channels:
the CLI markdown card (a trailing `Cache note:` line), the MCP
markdown card (same line, appended before MCP redaction), and search
JSON (`_meta.notes`). The gap is the fourth channel: single-entity
`get ... --json` bodies have no notes or `_meta` field to carry it —
they serialize the entity struct directly (for example
`src/cli/disease/dispatch.rs` `render_loaded_card` with
`json_output`).

## Why deferred

Adding a `_meta.notes` shape to every get-JSON body changes the
public JSON surface for all entities at once. That deserves its own
ticket with a schema decision (a shared wrapper versus per-entity
fields), not a rider on the wording fix. Both markdown paths and the
search JSON path are covered today; the get-JSON gap fails quietly:
the note simply does not appear there, the same as before ticket
1256.

## Revisit trigger

When a user or contract test asks for the stale age on a get-JSON
body, or when the flat-schema work (ADR 0002 follow-ups) next touches
the JSON envelope shape. Then: pick the envelope, add the notes
channel once, and reuse `crate::sources::take_stale_serve_sentences`
at the single build site.

## GWAS search, resolved 2026-09-28 (ticket 1263)

GWAS search JSON also cannot carry the stale-cache note, for a
different reason than get-JSON: the GWAS client sends every request
with `CacheMode::NoStore` (`src/sources/gwas.rs`, a deliberate
bypass after cache decode failures), so no stale serve exists to
describe. If that bypass is ever revisited, the notes channel added
for article and search-all is the pattern to copy.
