---
flow: build
priority: 7
deps: ["0957"]
---
# Enforce ClinGen input and timeout boundaries

Two narrow ClinGen paths bypass normal boundaries: the CSpec custom client has
no connect or total timeout, and ERepo's 64 KiB reader consumes a sentinel byte
without rejecting it. Fix both at their local transport/input seams.

## CSpec timeout contract

Use the shared provider policy of a 10-second connect timeout and 30-second
total request timeout for every CSpec request, including a caller-supplied
version IRI after URL-policy validation. Keep both durations injectable for
tests without changing production defaults. Ten seconds applies to each TCP
connect attempt. Thirty seconds applies to one complete HTTP request from
initial connect through headers and body, including redirects; this ticket adds
no retry, and middleware may not restart that deadline. A multi-request CSpec
operation gets the same policy independently for each request. A timeout is
attributed to ClinGen CSpec through the safe public error projection from 0909;
it never logs a raw URL, response body, or credential.

## ERepo input contract

Define one `MAX_EREPO_INPUT_BYTES = 65_536` used by stdin and file reads. Read
at most limit plus one, reject any extra byte before JSON parsing or transport,
and return the structured `input_too_large` error with `limit_bytes: 65536`.
Exactly 65,536 bytes may proceed to parsing. Whitespace after a valid document
counts toward the boundary and cannot turn a 65,537-byte input into a valid
request. A read error remains distinct from invalid JSON and oversize input.

## Done when

- An injected connector deterministically proves connect-timeout handling.
  Loopback servers separately stall before headers and during a body; tiny
  injected request deadlines fail with CSpec attribution and no sensitive
  detail.
- Redirected and exact-version CSpec requests retain the same timeout and URL
  policy; retries cannot extend one total request indefinitely.
- Exact-limit and limit-plus-one stdin/file fixtures cover valid JSON,
  trailing whitespace, multibyte bytes, short reads, and I/O failure, and prove
  no ERepo transport call on rejection.
- CLI JSON/Markdown errors, MCP execution, docs, and request-plan tests agree on
  the two boundaries.

## Authorized test changes

Design commits may restate the CSpec client builder, injected timeout seams,
ERepo input reader, local servers/files, safe errors, and ClinGen docs. Other
providers and general upload support are out of scope.

The src line ceiling may rise by at most 100 lines.
