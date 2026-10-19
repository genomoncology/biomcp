# Pathway recording fidelity

The normal pathway replay uses successful retained Reactome and WikiPathways observations. A separately named synthetic Reactome failure scenario preserves the four original search-all failure snapshots. Runtime, drift checker and primary source fixtures remain unchanged.

## Sources and projections

The branch starts at 3e4b29e92726d481ec6574727db8a8b19647a38a. Controlled checks use published main 1c4753a453569d17dd66a13eb3d312ab8a5af4d4 plus retained private patches. The seven primary observations came from job 2281/1792449561439-4540 between 2026-10-19 22:39:22 UTC and 22:39:34 UTC. This repair made no provider requests.

Each observation uses the exact manifest upstream URL, GET, no request body, User-Agent biomcp-cli/0.9.4, Accept */*, Cache-Control no-store and Accept-Encoding identity. Current source request plans and shared client headers agree with these application identities; transport differences remain as described in record 2281. The response metadata now records the observed Content-Type. WikiPathways ETag and Last-Modified also come from this observation. The missing historical BRAF and WikiPathways Content-Type values remain unknown. The legacy filenames do not establish the new observation date.

| Recording | Retained raw SHA256 | Public projection |
| --- | --- | --- |
| Reactome participants | f137b84a59d3504187a0e3ca37267f9db5c2b273b558b6acfce52a865df7299f | First ten displayName values; unchanged. |
| Reactome contained events | a5efd86a58da68ec058273e87920dcb12a3f166463da11d15545b94a5a9a57b8 | First ten displayName values and the first integer, 9660557, at raw index 13. |
| Reactome pathway detail | 5752c1c30df4468c366c9b65aada454cd0797c9d040a2e3c7dd23e6b14722295 | Existing stId, displayName, speciesName and summation text; unchanged. |
| Reactome BRAF search, limit 3 | 65c64d25360870762dad0ee0a293b32ffce6f6b7fc23d5b521d40da437ee194a | First result group's entries, retaining only observed stId, id and name; HTTP 200. |
| Reactome MAPK search, limit 3 | a72ff948372ad45d36e8f125c8006b4f2b07b2d4a4eeda69cb34c8a1ab93acd9 | Existing first group entries; remove absent totalResults. |
| Reactome MAPK search, limit 5 | a72ff948372ad45d36e8f125c8006b4f2b07b2d4a4eeda69cb34c8a1ab93acd9 | Existing first group entries; remove absent totalResults. |
| WikiPathways search | 96a7a766e931413db8827603389594456c4c5dfff057c577a45f5d805c626d2f | Existing JSON equals the observation; body unchanged. |

Reactome's full contained-events array has 69 objects and twelve integers. The minimal sample preserves its object/integer union without duplicating the full response. ReactomeContainedEvent already accepts both types; map_contained_events skips integer IDs. ReactomeSearchResponse already accepts an absent totalResults. Search projections retain the existing ten-entry first group because those entries serve the recorded command limits; raw MAPK replies contain additional groups. Projection checks compare selected values and ordering directly with retained raw JSON, in addition to the existing structural checker.

The reviewed source registry and licensing page identify Reactome database data and derived files as CC0 and WikiPathways content as CC0. No Reactome art, icons or image assets enter the projections. Full raw replies remain in the private primary experiment; no duplicate full payload was added.

## Separate failure scenario

The old BRAF HTTP 500 body moves to tests/fixtures/pathway/synthetic-reactome-failure/body.json. The four original search-all JSON and Markdown snapshots are copied unchanged under names ending in synthetic-reactome-failure. The new test explicitly injects HTTP 500 in its isolated temporary replay root through the existing replay_environment root control. This authored failure is absent from the normal replay manifest and therefore from default live drift comparisons. The normal four search-all snapshots instead record the observed successful pathway rows.

## Evidence

Job 2282/1792449744175-3788 reproduced the contained-events element type change and both missing totalResults fields against published main and retained raw replies. It also confirmed the selected participant and detail values and the full WikiPathways JSON equality.

Job 2282/1792449864318-55982 demonstrated exactly four failures when the retained successful BRAF reply met the old normal snapshots. After generating the approved success snapshots, all twenty affected strict replay tests passed, including four synthetic failure cases. All sixteen focused Reactome and WikiPathways source tests passed, and Ruff lint passed. Ruff formatting requested changes. An attempted manual wrap still failed formatting in job 2282/1792450044221-55907; that job also passed all seven comparisons and the same twenty replay and sixteen source tests. Final mechanical qualification applies Ruff formatting directly and verifies the Python syntax tree stays identical. This development job stopped at formatting and did not run repository lint. The controlled failure output and original candidate patch remain private.

Private experiment 2101-biomcp-2282-pathway-recording-fidelity retains the OWNER, plan, comparison script, source and patch identity, controlled failures, final patch and qualification logs. Experiment 2100 retains the unique full raw replies and original request evidence. Final qualification compares all seven status/header/structure pairs and source-value projections, verifies the four synthetic snapshots equal the original snapshots byte for byte, runs the affected strict replay and focused source checks, and runs repository lint. No full gate, provider access, runtime change or checker change forms part of this repair.

The projections qualify only these retained requests at their observation times. They sample consumed fields and rows rather than claiming full-payload fidelity or future provider stability. Missing historical headers remain unknown. The independent fixture repairs own other manifest rows and must be preserved when this branch lands.
