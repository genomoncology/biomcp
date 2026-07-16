# VAERS Queries

The VAERS slice of BioMCP is an aggregate vaccine-safety view, not a case-level
report browser. These canaries keep vaccine-first routing, aggregate-only
reporting, source-specific limitations, and combined/default behavior visible.

## Source Selection Contract

The adverse-event surface should keep the VAERS source switch visible in help so
users can tell when they are asking for FAERS, VAERS, or the combined path.

```bash
../../target/release/biomcp search adverse-event --help | mustmatch like '--source <faers|vaers|all>
biomcp search adverse-event "COVID-19 vaccine" --source all --limit 5
biomcp search adverse-event "MMR vaccine" --source vaers --limit 5'
```

## Vaccine-Only Truthfulness

If the user forces the VAERS source for a non-vaccine query, BioMCP should say
that plainly instead of pretending the source searched nothing.

```bash
../../tools/biomcp-ci search adverse-event --drug aspirin --source vaers | mustmatch like 'Status: query_not_vaccine
VAERS is vaccine-only; this query did not resolve to a vaccine identity.'
```

## Positive VAERS Aggregate Live Canary
<!-- mustmatch-lint: skip -->

VAERS is a live CDC WONDER aggregate source. The verify lane must prove at least
one realistic vaccine query reaches the positive aggregate path; negative and
help-only assertions are not enough to support a release claim.

```bash run id=mmr-vaers-positive-live exit=0 timeout=180
../../tools/biomcp-ci search adverse-event "MMR vaccine" --source vaers --limit 5 | mustmatch like 'CDC VAERS Summary
Matched vaccine: MMR
CDC WONDER code: MMR
Serious reports:
Non-serious reports:
Age distribution
Top reactions
Source: CDC VAERS'
```

## Typed combined-source outcomes

Combined search keeps FAERS and VAERS outcomes independent while preserving the
existing VAERS aggregate status. Either source can report bounded uncertainty
without erasing the other source's result.

```bash
../../tools/biomcp-ci --json search adverse-event "MMR vaccine" --source all --limit 5 \
  | jq -r '.vaers.status, .section_outcomes.faers.outcome, .section_outcomes.vaers.outcome' \
  | mustmatch '/^(ok|empty|unavailable)\n(data|empty|degraded|unavailable)\n(data|empty|degraded|unavailable)/'
```

## Source-Specific Limitations

FAERS-style filters should fail truthfully when the user forces the VAERS
source, instead of being silently ignored.

```bash
../../tools/biomcp-ci search adverse-event --drug 'COVID-19 vaccine' --source vaers --outcome death 2>&1 | mustmatch like '--source vaers only supports
unsupported flags: --outcome'
```

## FAERS Count Field Validation

The `--count` option is for openFDA field aggregations. The overall report total
lives in response metadata, so `total` should be rejected instead of forwarded as
a fake field that returns empty buckets.

```bash
../../tools/biomcp-ci search adverse-event --drug pembrolizumab --count total 2>&1 | mustmatch like '--count total is not a count field
patient.reaction.reactionmeddrapt'
```
