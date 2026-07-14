# ORCID Public API Contract and Operational Decision

Status: internal source foundation only. Reviewed and probed 2026-07-14.

## Decision

BioMCP may use anonymous ORCID Public API v3.0 record and works reads as an
internal, unexposed source seam. Every request is anonymous, forced `NoStore`,
body-bounded, and paced to one request per 100 ms in a long-lived process. BioMCP
must not expose this seam through CLI or MCP until a later ticket establishes an
applicable deployment and terms basis.

The [Public API terms](https://info.orcid.org/public-client-terms-of-service/)
limit the free Public API to non-commercial services and define revenue-generating
use as commercial. If those terms do not fit a deployment, BioMCP keeps only
ORCID evidence already supplied by citations. It does not fall back to member,
private, OAuth, or credentialed access.

## Access and representation

ORCID distinguishes the anonymous/registered Public API from member access. The
[Public API overview](https://info.orcid.org/what-is-orcid/services/public-api/)
says it retrieves public record data; private and trusted data are outside this
contract. The [record schema](https://info.orcid.org/documentation/integration-guide/orcid-record/)
documents item visibility, put codes, source fields, and public/private behavior.
BioMCP maps only entries whose visibility is exactly `public`, matching the v3
JSON representation, and only the narrow professional fields required by the
author-identity architecture.

Record and works plans use v3.0 paths and
`Accept: application/vnd.orcid+json`. The official
[works tutorial](https://github.com/ORCID/ORCID-Source/raw/refs/heads/main/orcid-api-web/tutorial/works.md)
documents the vendor media type and `/works` resource. Successful responses with
missing or different media types are rejected before decoding.

ORCID iDs are validated with ISO 7064 MOD 11-2, including an `X` check digit, as
documented in [Structure of the ORCID Identifier](https://support.orcid.org/hc/en-us/articles/360006897674-Structure-of-the-ORCID-Identifier).
Deprecated/merged IDs can return 301 and a canonical `Location`; ORCID's
[API error documentation](https://github.com/ORCID/ORCID-Source/blob/development/orcid-api-web/tutorial/api_errors.md)
documents that behavior. BioMCP follows only same-origin redirects, retains both
requested and canonical IDs, and requires the final URL to agree with the
decoded record ID or top-level works path.

## Bounds, paging, and status truth

The v3 `/works` resource exposes no page parameters. BioMCP sends one bounded
request, returns no continuation, and does not send invented `offset` or `limit`
values. ORCID's [works guidance](https://support.orcid.org/hc/en-us/articles/360006973133-Add-works-to-your-ORCID-record)
sets a record ceiling of 10,000 works; BioMCP's independent response-body limit
remains 8 MiB. An oversized body is an explicit
unavailable outcome, never a falsely complete works collection.

Search is a separate API and is not implemented here. The official
[search tutorial](https://info.orcid.org/documentation/api-tutorials/api-tutorial-searching-the-orcid-registry/)
documents `start`/`rows`, a maximum of 1,000 rows per response, and the Public API
10,000-result ceiling. Those search semantics must not be projected onto `/works`.

Final 404, 429 (with only a bounded `Retry-After` value), and 5xx responses map
to distinct not-found, rate-limited, and unavailable outcomes. Wrong media,
malformed JSON, rejected redirects, and inconsistent identity are errors. No
non-success response becomes an empty record or works list.

## Rate, cache, and process model

ORCID's [API quota FAQ](https://info.orcid.org/ufaqs/what-are-the-api-limits/)
lists anonymous access at 12 requests/second sustained, burst 40, and 25,000
reads/day/IP; registered Public API access has a separate 100,000/day/client
allowance. BioMCP's named `orcid` policy uses a conservative 100 ms minimum
interval (10 requests/second). This limiter is process-local and cannot enforce
the daily IP quota across processes. Deployments needing aggregate control use
the existing long-lived service/process model or external coordination.

Every ORCID request carries an explicit middleware `NoStore` override, including
when `BIOMCP_CACHE_MODE=infinite`. Provider cache headers are evidence, not the
enforcement mechanism.

## Provenance and privacy boundary

ORCID distinguishes the source that added an item from the assertion origin that
created the iD-item connection; see
[ORCID source guidance](https://support.orcid.org/hc/en-us/articles/360022948733-Where-can-I-see-the-source-of-information-on-my-record).
BioMCP preserves ordinary source ORCID/name, assertion-origin ORCID/name, put
code, visibility, created/modified dates, structured partial dates, and raw plus
normalized external IDs. It preserves every public work summary in a group and
does not deduplicate provider assertions.

The wire and mapped DTOs contain no email, researcher/homepage URL, biography,
keywords, standalone address/demographic location, gender, ethnicity, or inferred
demographic field. Organization locality is retained only inside a sourced
employment organization.

## Redacted probe observations (2026-07-14)

The probes used public professional records and retained no response payload,
email, homepage, private data, or unnecessary personal data in the repository.

- Anonymous record and works GETs returned 200 with
  `application/vnd.orcid+json;charset=UTF-8`.
- Responses included `Cache-Control: no-cache, no-store, max-age=0,
  must-revalidate`; BioMCP still forces its own no-store request mode.
- A public record exposed public name/employment source, assertion origin,
  put-code, visibility, and date values.
- A works response grouped duplicate provider assertions under external IDs;
  each summary retained its own provenance.
- `/works` probes with `offset=-1`, `offset=999999`, `limit=0`, and `limit=1001`
  returned the same complete body, confirming those parameters are ignored and
  must not be represented as paging.
- Anonymous search with `rows=1001` returned HTTP 400 error 9012, consistent with
  the documented 1,000-row maximum. Search remains out of runtime scope.
- A merged-ID live example was not sought or stored; same-origin 301 handling is
  supported from the official contract and exercised with deterministic fixtures.
