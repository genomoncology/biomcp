---
flow: build
priority: 6
---

# Open an exact author record and claimed works by ORCID

BioMCP accepts exact Semantic Scholar author identifiers, but it rejects ORCID identifiers. A researcher knowledge-base run had to call ORCID outside BioMCP to establish each researcher’s claimed works. Name search could not replace that evidence because it selected a wrong same-name person in one case and found two valid Semantic Scholar profiles in another. The current behavior and provider evidence appear in `sdlc/issues/feature-support-orcid-as-an-exact-author-provider.md`.

ORCID’s version 3.0 Public API exposes public person, record, and works endpoints. Live works requests for `0000-0002-1678-5864` and `0000-0002-5561-6932` returned 176 and 222 work groups during verification on 2026-09-04. ORCID documents Public API credentials and a public-read token as the supported access method. Anonymous requests happened to work during verification, but BioMCP cannot rely on that behavior as the provider contract.

## Required behavior

`biomcp get author orcid:<id>` returns the researcher’s public ORCID record as an exact provider record. `biomcp author papers orcid:<id>` returns the public works claimed on that record and supports continuation through the corpus. Both responses preserve useful publication identifiers, identify ORCID as the source, report source availability truthfully, and provide runnable next commands.

BioMCP uses ORCID’s supported Public API authentication. Health output tells an operator when required ORCID access is missing or unusable. Invalid ORCID identifiers fail before provider work.

ORCID and Semantic Scholar records remain separate unless a later feature proves a link. BioMCP does not merge authors by name, affiliation, topic, or overlapping search results.

Done, observably:

- A valid public ORCID opens an exact author card and pages through that record’s claimed works.
- A claimed work exposes its available DOI, PMID, or other stable identifier for an article follow-up.
- A later page can be requested from the continuation returned by the prior page.
- Human-readable and JSON responses identify the provider and distinguish empty, unavailable, and failed results.
- Missing credentials, malformed identifiers, and provider failures produce actionable errors without exposing secrets or raw provider failures.
- Existing Semantic Scholar author commands keep their current behavior.

Boundary: this ticket covers public ORCID author records and claimed works. It does not resolve identity across providers, search ORCID by name, read restricted ORCID data, create a BioMCP person identifier, or treat a claimed work as proof that two provider records describe the same person.
