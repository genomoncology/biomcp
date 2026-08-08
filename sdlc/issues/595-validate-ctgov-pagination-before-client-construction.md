# Validate CTGov pagination before client construction

Severity: nice-to-have.

Carried over from March, where it was raised against ticket 595
on 2026-07-19 and left open. The text
below is as filed.
## Summary

The public CTGov `search_page` path constructs `ClinicalTrialsClient` before its
injected-client helper validates `--limit`, `--offset`, and `--next-page`.

## Detail

`src/entities/trial/search/mod.rs::search_page` now validates trial filters before
client construction, but `validate_search_page_args` remains inside
`search_page_with_ctgov_client`. Invalid pagination therefore performs avoidable
client setup before returning `invalid_argument`. This predates ticket 595 and does
not bypass that ticket's numeric guards, so it was not changed during review.

## Suggested action

Call the existing pagination validator at the public entity boundary before
`ClinicalTrialsClient::new`, retaining defensive validation in the injected-client
helper. Add a focused native ordering regression; do not alter shipped behavior.
