---
flow: build
priority: 9
---
# Prove RequestPlan execution over a local transport

Provider tests strongly cover pure `RequestPlan` construction and production
decoders, but there is no single contract proving that `request_from_plan`
sends those pure values as the intended HTTP request and that the standard
send, body, and decoder seams compose correctly. Source-conversion tickets
must not each invent a different substitute for this missing boundary.

## Shared contract

One reusable loopback fixture observes the production request built by
`request_from_plan`. Tests cover:

- GET and POST paths, leading-slash normalization, ordered and repeated query
  parameters, and percent encoding;
- duplicate-safe headers plus text, form, and JSON bodies;
- redirect policy, connect/total timeout classification, and source-attributed
  transport errors;
- declared-length and chunked response-body limits; and
- a complete successful path through the standard body reader and
  `decode_json`, plus HTTP-status and malformed-response failures.

The contract invokes production request, middleware, reader, and decoder code.
It must not inspect source text, build a second test-only request model, or use
public network. Provider clients with custom authentication, URL policy, or
cache behavior add only a focused test for that unique behavior and continue
to rely on this common proof for the rest.

## Done when

- A deliberately repeated query key and each body type are observed exactly at
  the loopback server.
- Success bytes reach the production decoder unchanged.
- Redirect, timeout, status, and size failures retain safe provider context.
- The source-conversion ticket family can cite this test instead of duplicating
  generic executor coverage.

## Authorized test changes

Design commits may restate and extend the test modules in `src/sources/mod.rs`
that currently cover the response middleware, body readers, retry/status
mapping, and `RequestPlan` helpers. Representative source construction tests
may be changed only to consume the shared fixture; provider-specific plan and
decoder assertions remain intact.

The src line ceiling may rise by at most 230 lines.
