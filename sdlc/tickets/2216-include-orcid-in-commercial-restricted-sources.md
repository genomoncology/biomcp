---
priority: 1
---

# 2216 — Include ORCID in commercially restricted sources

Status: OPEN.

Milestone: 0.9.4

Reviews: revision b8bf623, accept

## Outcome

The source registry describes the non-commercial terms for ORCID Public and Anonymous APIs, and commercial-restricted source selection includes ORCID so users can disable that direct API under the existing policy.

- Evidence: 2204 review and coordinator independent verification of official ORCID Public APIs Terms section2, https://info.orcid.org/public-client-terms-of-service/ . The terms restrict API use in revenue-generating products/services; public-data CC0 does not remove the API-service restriction. Current sources.json has commercial_terms:null and links an integration overview. PM ruled priority1 in0.9.4 and specified a registry correction plus a group-membership test. WHO and Semantic Scholar remain unverified and outside this fix.
- Changes: `docs/reference/sources.json` `docs/reference/source-licensing.md` `CHANGELOG.md` and the existing source-registry/group contract tests. Declare exact test paths before edits. Keep the group derived from registry metadata; do not add a second hardcoded source list or change default source availability.
- Proof: reproduce the current commercial-restricted group omitting ORCID with the existing actual selection path; an outside-in/group contract fails before correction and passes afterward. Confirm explicit ORCID and group disable retain truthful disabled reasons and avoid its fixture requests. Verify source registry/licensing documentation checks. Targeted build-job tests, fresh review and ordinary landing/main checks follow.

PM decision: Correct the registry and test group membership; no provider contact, license purchase, default-disable policy change or claim about unverified providers is authorized.

## Progress

- 2026-10-18 started

Coordinator pre-code scope: exact test claims `tests/disable_sources_cli.rs` and `tests/test_source_licensing_docs_contract.py`, in addition to the three named registry/docs/changelog files. The current actual parse_disable_spec derives groups from commercial_terms metadata, and ORCID's null row therefore omits it. Add a dedicated real-CLI group case covering ORCID health membership/truthful group reason plus exact author and author-papers refusal with zero local fixture requests; extend the existing explicit-disable health alias table for ORCID, preserving KEGG/Enrichr cases. Add the official API terms/category assertion to the existing registry/docs contract. Capture that owning case RED with the old registry before correcting the entry, then targeted GREEN. No auth/default-availability or second runtime list change. Priority1 execution may proceed through the build job; at most two own build jobs are active, with2199 as the other slot.
- 2026-10-18 next: Actual old-registry CLI RED2216/1792349722483-57186 reproduced ORCID missing from commercial group: health probed fixture and errored instead of disabled,66.978s. Registry/docs correction and owning regressions passed GREEN2216/1792349825275-82878 in126.483s: both affected CLI cases, licensing-docs module, formatting, Clippy warnings denied, Ruff, quality and diff checks. Developer stopped clean and pushed once at b8bf623174562efb2cb17a119838067d68621d3d; fresh review running. Group remains registry-derived, explicit disable is covered, defaults/auth unchanged.
