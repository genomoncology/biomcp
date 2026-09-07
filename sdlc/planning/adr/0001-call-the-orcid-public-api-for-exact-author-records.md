# 0001 — Call the ORCID Public API for exact author records

- Status: accepted
- Date: 2026-09-07
- Supersedes: the "no ORCID API calls; ORCID is citation-supplied evidence only" product boundary cited in record 0581

## Context

BioMCP accepts exact Semantic Scholar author identifiers and rejects ORCID identifiers. A research run had to call ORCID outside BioMCP to establish which works a researcher claims. Name search cannot substitute. It selected the wrong same-name person in one case and returned two valid profiles in another.

Record 0581 retired an orphaned ORCID client from `src/sources/orcid.rs` because executable network code contradicted a decided product boundary. That boundary held that ORCID is evidence supplied through citations and that BioMCP makes no ORCID requests of its own. The cleanup was correct under the boundary of the day. The boundary itself is what this decision changes.

## Decision

BioMCP calls the ORCID version 3.0 Public API to serve `biomcp get author orcid:<id>` and `biomcp author papers orcid:<id>`.

Access uses a pre-issued public-read bearer token supplied in `ORCID_ACCESS_TOKEN`. BioMCP does not perform an OAuth exchange, refresh a token, persist one, or log one.

## Options weighed

**Keep the boundary.** Costs the capability outright. A caller who needs claimed works keeps leaving BioMCP to get them, and rebuilds pagination, retry, provenance, and normalization outside the tool. Nothing else is gained; the boundary was never protecting against a cost anyone has named.

**Call the API anonymously.** ORCID documents an anonymous tier, so this is a real contract rather than an accident. It costs a lower ceiling of 25,000 reads per day counted per IP address. A shared address or a CI runner consumes someone else's budget invisibly.

**Call the API with Public API credentials (chosen).** Costs one environment variable that an operator must obtain and set, plus the health reporting to say when it is missing. Buys a 100,000 reads per day ceiling counted per Client ID, so one deployment's usage cannot be spent by an unrelated caller sharing an address.

## Why the limits do not bind

ORCID publishes its quotas. The anonymous tier allows 12 requests per second with a burst of 40 and 25,000 reads per day per IP address. The Public API with credentials allows the same rate and 100,000 reads per day per Client ID. The Member API allows 24 requests per second with no quota.

Source: https://info.orcid.org/ufaqs/what-are-the-api-limits/

The Public API is offered to organizations that are not ORCID members. Credentials require an ORCID account with a verified email and agreement to the Public API terms. No membership fee applies.

Source: https://info.orcid.org/documentation/features/public-api/

A works page returns at most 100 groups per request. Verification on 2026-09-04 found 176 and 222 work groups for two real researchers, so a complete author fetch costs two or three requests. Reaching the daily ceiling would take roughly forty thousand full researcher lookups.

## Cost accepted

BioMCP gains a network dependency on a provider it previously did not call. A new credential can be missing, malformed, or rejected, so three failure states now exist that did not before. Ticket 1142 pins each one to an exact message, exit code, and recovery line.

## What this does not change

Record 0581's dead-code cleanup policy stands. The deleted orphan client is not reinstated; ticket 1142 specifies a newly reviewed client with bounded public surfaces.

Semantic Scholar's `externalIds.ORCID` remains untrusted and excluded. ORCID and Semantic Scholar author records stay separate unless a later feature proves a link. Every existing `orcid_link_not_established` warning and all non-linkage behavior are unchanged.

## Reversal

Overturning this decision means writing a superseding ADR, retiring ticket 1142, and removing the source module. The credential stays optional throughout, so an operator who sets no token sees the same refusal shape BioMCP already gives for other absent keys.
