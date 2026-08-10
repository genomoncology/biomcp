---
flow: build
priority: 5
---
# Surface a criteria specification's file attachments

## Command contract

Add an opt-in manifest flag to the existing command shape:

    biomcp gene cspec PTEN --version <resource-iri> --files
    biomcp gene cspec PTEN --capture-id <capture-id> --files

The second form reads the selected stored capture and performs no provider
request. Do not add a trailing files pseudo-section that Clap cannot parse
consistently with the existing document subcommand.

## Done when

The files view lists each public File entity from the selected CSpec payload:
label, filename, declared media type, declared size when present, resolved
same-origin download URL, and stable attachment identifier. Normal criteria
output includes attachment_count so a caller knows files exist without asking
for the manifest.

This ticket lists metadata only; it does not download attachments.

## Safety and provenance

- Parse only File entities actually linked from the selected specification.
- Reject or mark non-public, cross-origin, malformed, duplicate, or
  unsupported file URLs.
- Accept at most 100 linked File entities. A label, filename, declared media
  type, or stable attachment identifier may contain at most 512 UTF-8 bytes;
  a resolved download URL may contain at most 4,096 UTF-8 bytes. Check these
  limits before cloning the value into the response.
- An excess count or oversized string fails the files view with a typed,
  source-attributed provider-response-limit error. Do not truncate an
  identifier or URL, return a partial manifest, or silently omit the excess
  attachment. Normal criteria output may still report the exact linked
  `attachment_count` without materializing the manifest.
- Preserve capture_id, sha256, resource IRI, and capture binding in JSON and
  human output.
- A --capture-id request must not refetch or silently select another version.

## Proof required

Use a real receipted PTEN GN003 payload that contains the attachment entities,
plus synthetic malformed-edge fixtures. Pin Clap grammar, production parsing,
no-refetch capture reuse, URL validation, compact JSON, and readable output.
Synthetic cases cover exactly 100 and 101 attachments, every exact string
boundary and boundary plus one, and prove that a rejected manifest contains no
partial rows.

## Authorized test changes

Design commits may restate CSpec CLI parsing, parser/capture fixtures,
gene-cspec rendering tests, schemas/examples, and the selected real capture.
Mechanical construction fixes may land with implementation while existing
criteria assertions remain unchanged.

Cross-specification search remains out of scope and stays in draft 0894.

The src line ceiling may rise by at most 180 lines.
