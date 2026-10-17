# ORCID fixtures

## `person_0000-0002-1825-0097_20261006.json`

Captured from `https://pub.orcid.org/v3.0/0000-0002-1825-0097/person` on 2026-10-06. The demo record for Josiah Carberry. Provider-faithful capture of the public person document, including the `visibility` values the name projection gates on.

## `person_0000-0002-4299-3018.json` and `works_0000-0002-4299-3018.json`

Captured from `https://pub.orcid.org/v3.0/0000-0002-4299-3018/person` and `.../works` on 2026-10-16 for ticket 2088, with no `Authorization` header: the tokenless public reads this ticket pins. The works capture is the full reply, 380 groups, not trimmed, so the group representative selection and the deduplication upstream run on real provider shapes.
