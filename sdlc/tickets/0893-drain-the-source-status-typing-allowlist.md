---
flow: build
priority: 3
---
# Drain the source-status typing allowlist

## Done when

`tools/source-status-typing-allowlist.json` is empty and the ratchet
still passes, so no struct pairs a source-attributing field with a
free-form `status: String`. If draining an entry is not worth it, the
body says which and why, and the allowlist keeps only those.

## The finding

Raised under March and carried over when BioMCP moved to the sdlc
factory. The text below is as filed.

Ticket 634 adds `check_source_attributed_status_is_typed` to
`tools/check-quality-ratchet.py`, which stops any *new* `Serialize` struct from
pairing a source-attributing field with a free-form `status: String`. It converts
`VariantArticleSourceStatus`, the struct that caused the misattribution, and
seeds `tools/source-status-typing-allowlist.json` with the three pre-existing
offenders so the check can land without a scope explosion.

This issue is the drain. Each allowlist entry should be converted to a typed
enum and removed:

| Struct | File |
|---|---|
| `HealthRow` | `src/cli/health/mod.rs` |
| `CanonicalEquivalenceObservation` | `src/entities/article/variant_search.rs` |
| `VariantArticleProviderPlan` | `src/entities/article/variant_search.rs` |

## Why it matters

These are the structs where *we* assert a judgement about a source, as opposed to
relaying a value the source gave us. A free-form `String` there is what let the
`exact_lexical` route report `semanticscholar: unavailable` for a call that was
never made — the vocabulary had no rung for "not attempted", so the code had to
pick a wrong word. Every entry left on the allowlist is a place that can still
happen.

`HealthRow` is the most exposed of the three: it is the surface operators read
first when deciding whether a provider is at fault, which is exactly the decision
this class of bug corrupts.

## Explicitly not offenders

Structs that deserialize a provider's own status verbatim — `Trial`,
`TrialSearchResult`, `CivicEvidenceItem`, `CivicAssertion`, `EmaDrugSearchResult`,
`EmaRegulatoryRow`, `RecallSearchResult`, `CanonicalEquivalence` — are correctly
typed as `String`. They relay an upstream value rather than assert our own
judgement, and the ratchet must keep not firing on them. Do not "fix" these.

## Suggested action

One small ticket per struct, or one ticket for all three if the enums turn out to
share a shape. Each conversion must keep the serialized JSON values unchanged
(these are agent-facing contracts) and must remove its allowlist entry in the
same change, so the ratchet only ever shrinks.
