---
priority: 1
---

# 2216 — Include ORCID in commercially restricted sources

Status: OPEN.

Milestone: 0.9.4

## Outcome

The source registry describes the non-commercial terms for ORCID Public and Anonymous APIs, and commercial-restricted source selection includes ORCID so users can disable that direct API under the existing policy.

- Evidence: 2204 review and coordinator independent verification of official ORCID Public APIs Terms section2, https://info.orcid.org/public-client-terms-of-service/ . The terms restrict API use in revenue-generating products/services; public-data CC0 does not remove the API-service restriction. Current sources.json has commercial_terms:null and links an integration overview. PM ruled priority1 in0.9.4 and specified a registry correction plus a group-membership test. WHO and Semantic Scholar remain unverified and outside this fix.
- Changes: `docs/reference/sources.json` `docs/reference/source-licensing.md` `CHANGELOG.md` and the existing source-registry/group contract tests. Declare exact test paths before edits. Keep the group derived from registry metadata; do not add a second hardcoded source list or change default source availability.
- Proof: reproduce the current commercial-restricted group omitting ORCID with the existing actual selection path; an outside-in/group contract fails before correction and passes afterward. Confirm explicit ORCID and group disable retain truthful disabled reasons and avoid its fixture requests. Verify source registry/licensing documentation checks. Targeted build-job tests, fresh review and ordinary landing/main checks follow.

PM decision: Correct the registry and test group membership; no provider contact, license purchase, default-disable policy change or claim about unverified providers is authorized.

## Progress

- 2026-10-18 started
