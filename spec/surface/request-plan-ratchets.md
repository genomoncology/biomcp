# Request-Plan Ratchets

BioMCP keeps source request construction deterministic before any live upstream call.
These ratchets now live in the language-native and Python/static contract lanes
instead of the routine Markdown spec gate. The remaining executable examples in
this document are user-facing help and documentation canaries.

## Update Help Keeps Unsafe Checksum Override on the Option Stanza

The update command's unsafe checksum escape hatch must be proven against the
rendered option stanza, not only against prose elsewhere in long help. The
Python docs contract runs the rendered CLI help and extracts the actual option
block.

## MyDisease Rejects Path and Query Separators Before Network

A disease ID is data, not a path fragment. The no-network Rust ratchet must
prove that slash, backslash, query, and fragment separators are rejected while a
valid ontology ID still plans the `/disease/{id}` request shape.

## Request Commands Consume Captured Fields at Execution Boundaries

Command dispatch should not construct request structs that executors ignore.
The Rust seam tests prove discover, disease search, disease fallback, and
article dispatch consume the request fields that carry user intent into source
or backend calls.

## PubMed and PubTator Consume Planned Auth and Cache Modes

Secret-aware article sources must use the plan's redacted auth/cache modes at
the executor boundary. These tests use synthetic keys and keyless clients so the
routine gate proves keyed behavior without requiring real credentials.

## Shared Retry-After Waits Stay Bounded

Shared HTTP retries should honor ordinary upstream `Retry-After` hints without
letting an extreme header park a CLI command or March worker indefinitely. The
Rust policy tests keep normal, malformed, extreme, and total-budget paths
deterministic without calling a live service.

## Ticket 401 Surface Ratchets

The post-migration spec runner keeps routine specs Markdown-only. The static
ratchets around spec quality and fixture realism live under `tests/surface/`,
where `make test` runs them without calling public services.

## Ticket 405 Architecture and Operator Contracts

Current repo docs must describe the shipped BioMCP architecture and operator
contracts, not migrated targets. The static contract suite keeps the routine spec
lane honest about the Rust crate surface, spec/surface participation,
cache/logging configuration, article fulltext dependencies, next-command
ownership, and docs navigation without calling public services.
