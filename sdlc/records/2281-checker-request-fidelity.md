# Replay checker request fidelity

The checker uses the current package version from Cargo.toml for its BioMCP User-Agent. The shared client construction in src/sources/mod.rs sets that identity and Cache-Control: no-store without a cache. Its default Accept is */*. Recorded Accept and Content-Type override the defaults after the existing validation; header names are normalized before merging. Recorded headers cannot override application identity or cache policy.

The checker retains Accept-Encoding: identity to compare bounded, uncompressed bytes. The Rust client enables gzip. Python and Rust still differ in TLS, HTTP, connection handling and other transport defaults. Matching application headers does not establish identical network fingerprints or identify which setting caused the earlier denials.

## Controlled evidence

All work builds on 3d0916841624ad39a8c83d929f51aa18099c0ed5. Tests and checks ran through build jobs against that published base with retained private patches. The controlled HTTP consumer observed Python-urllib/3.12 before the fix and biomcp-cli/0.9.4 afterward. The new outside-in cases check default and explicitly recorded Accept, versioned identity, no-store and identity encoding.

- Job 2281/1792449416798-65323 failed both new cases on the incorrect User-Agent before implementation.
- Job 2281/1792449434355-71721 passed all 31 focused replay drift tests and Ruff lint. Formatting identified a tuple wrap, which was corrected without changing behavior.
- Job 2281/1792449515653-91782 passed Ruff lint, Ruff formatting and the existing quality ratchet, including file and function boundaries.

The existing credential refusal, request reconstruction, provider restrictions, response cap, serial pacing, redirect refusal, proxy refusal and retry behavior remain in place. No transport dependency was added.

## Authorized recheck

Job 2281/1792449561439-4540 sent each of the seven previously denied recorded GET requests once between 2026-10-19 22:39:22 UTC and 22:39:34 UTC. Every response was HTTP 200 JSON. No Semantic Scholar, KEGG or other provider request ran. No credentials were inherited by the collection process, inspected or sent. The collector reused the checker's request validation, headers, pacing and response cap, disabled proxies and redirects, and added no retries. A whole-request deadline bounded each attempt.

The collection used the same checked code patch. Private evidence in experiment 2100-biomcp-2281-checker-request-fidelity retains the base revision, patch and hash, collection script, exact method and URL, explicit request headers, empty GET bodies, timestamps, all response headers, raw response bytes, hashes and comparison output. Raw responses remain private. The collection process exiting successfully means collection completed; it does not qualify these recordings.

| Recording | Exact manifest key | Observation |
| --- | --- | --- |
| 34 | `GET /reactome/data/participants/R-HSA-5673001` | Existing status, content type and structural paths match; additional paths are reported. |
| 35 | `GET /reactome/data/pathway/R-HSA-5673001/containedEvents` | Structural drift: changed type at $/*. |
| 36 | `GET /reactome/data/query/R-HSA-5673001` | Existing status, content type and structural paths match; additional paths are reported. |
| 37 | `GET /reactome/search/query?pageSize=3&query=BRAF&species=Homo+sapiens` | Live status 200 differs from recorded 500. Historical Content-Type is absent, so structural qualification remains unverified. |
| 38 | `GET /reactome/search/query?pageSize=3&query=MAPK+signaling+pathway&species=Homo+sapiens` | Structural drift: missing $/totalResults. |
| 39 | `GET /reactome/search/query?pageSize=5&query=MAPK+signaling+pathway&species=Homo+sapiens` | Structural drift: missing $/totalResults. |
| 43 | `GET /wikipathways/findPathwaysByText.json` | Search now returns valid JSON. Historical Content-Type is absent, so structural qualification remains unverified. |

| Private raw file | Bytes | SHA256 |
| --- | --- | --- |
| 034-reactome.body | 244106 | f137b84a59d3504187a0e3ca37267f9db5c2b273b558b6acfce52a865df7299f |
| 035-reactome.body | 30885 | a5efd86a58da68ec058273e87920dcb12a3f166463da11d15545b94a5a9a57b8 |
| 036-reactome.body | 23877 | 5752c1c30df4468c366c9b65aada454cd0797c9d040a2e3c7dd23e6b14722295 |
| 037-reactome.body | 97122 | 65c64d25360870762dad0ee0a293b32ffce6f6b7fc23d5b521d40da437ee194a |
| 038-reactome.body | 75068 | a72ff948372ad45d36e8f125c8006b4f2b07b2d4a4eeda69cb34c8a1ab93acd9 |
| 039-reactome.body | 75068 | a72ff948372ad45d36e8f125c8006b4f2b07b2d4a4eeda69cb34c8a1ab93acd9 |
| 043-wikipathways.body | 2346695 | 96a7a766e931413db8827603389594456c4c5dfff057c577a45f5d805c626d2f |

The checked patch SHA256 is 11297c1c6ab815109f728d7cf76763ba77d7e30fa75c6c168d00d1b26db6bea1. The artifact folder also retains failed formatting attempts; only the final green patch was used for provider requests.

Read-only inspection in job 2281/1792449632802-27875 found that containedEvents changes from ten object elements to a mixed array of 69 objects and twelve integers. This is an observed element-type union change, independent of changing identifier values. The existing ReactomeContainedEvent enum accepts objects and integer IDs; map_contained_events skips integer IDs. This evidence does not establish whether the mapped display names changed. WikiPathways has matching JSON structural paths and types when both payloads are interpreted as JSON, with no added paths; the absent historical Content-Type still prevents a full checker qualification. Neither inspection made a network request.

The consumer paths are ReactomeClient::participants, contained_events, get_pathway and search_pathways in src/sources/reactome.rs, and WikiPathwaysClient::search_pathways in src/sources/wikipathways.rs. The pathway sections in src/entities/pathway.rs consume participants and contained_events. ReactomeSearchResponse treats totalResults as optional, so that field's disappearance changes available total metadata rather than proving search decoding fails. The checker correctly reports its recorded structural disappearance.

The repository source registry identifies Reactome database data and derived files as CC0. Reactome art and icons have separate CC BY 4.0 terms. WikiPathways content is CC0. See docs/reference/sources.json and docs/reference/source-licensing.md for the existing reviewed source terms. Retained JSON bodies are private evidence; no art assets or provider credentials were collected. The separate fixture lane must preserve the applicable source terms when adopting these replies.

The successful recheck establishes access with this request profile at the observed times. It does not prove User-Agent alone caused the prior denials. Fixture payloads and manifest metadata remain unchanged. The missing historical content types cannot be reconstructed from the earlier denial replies, and no new manifest status schema was introduced. Payload drift and missing recording metadata need separate treatment before release qualification.
