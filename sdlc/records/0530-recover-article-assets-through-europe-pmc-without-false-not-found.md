---
base: f3a320f62d81aa47d5a33451714946b31d9de9c5
head: 582e8a492b10ced34d83b49bc27503fa460007af
---
`biomcp --no-cache --json get article 38821914 assets` resolves PMID 38821914 to PMC11143360, receives a PMC OA manifest, then follows the advertised archive URL to an HTTP 404. BioMCP discards that source error and returns `not_found`, falsely telling the caller that the article has no supported assets. On 2026-07-14 the Europe PMC `PMC11143360/supplementaryFiles` endpoint returned a valid 204,916-byte ZIP containing seven files, including `41408_2024_1068_MOESM1_ESM.docx`. This is a live false negative on an agent-facing evidence path.

Imported from March ticket 530. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/530-recover-article-assets-through-europe-pmc-without-false-not-found
