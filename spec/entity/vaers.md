# VAERS Queries

The VAERS slice of BioMCP is an aggregate vaccine-safety view, not a case-level
report browser. These canaries keep vaccine-first routing, aggregate-only
reporting, source-specific limitations, and combined/default behavior visible.

## Source Selection Contract

The adverse-event surface should keep the VAERS source switch visible in help so
users can tell when they are asking for FAERS, VAERS, or the combined path.

```bash
out="$(../../target/release/biomcp search adverse-event --help)"
echo "$out" | mustmatch like "--source <faers|vaers|all>"
echo "$out" | mustmatch like 'biomcp search adverse-event "COVID-19 vaccine" --source all --limit 5'
echo "$out" | mustmatch like 'biomcp search adverse-event "MMR vaccine" --source vaers --limit 5'
```

## Vaccine-Only Truthfulness

If the user forces the VAERS source for a non-vaccine query, BioMCP should say
that plainly instead of pretending the source searched nothing.

```bash
out="$(../../tools/biomcp-ci search adverse-event --drug aspirin --source vaers)"
echo "$out" | mustmatch like "Status: query_not_vaccine"
echo "$out" | mustmatch like "VAERS is vaccine-only; this query did not resolve to a vaccine identity."
```

## Source-Specific Limitations

FAERS-style filters should fail truthfully when the user forces the VAERS
source, instead of being silently ignored.

```bash
out="$(../../tools/biomcp-ci search adverse-event --drug 'COVID-19 vaccine' --source vaers --outcome death 2>&1 || true)"
echo "$out" | mustmatch like "--source vaers only supports"
echo "$out" | mustmatch like "unsupported flags: --outcome"
```
