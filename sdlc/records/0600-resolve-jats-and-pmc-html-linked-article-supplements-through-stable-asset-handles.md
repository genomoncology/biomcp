---
base: 961e36fed84737d62acf0385ac04941650c79c33
head: 9446b2acf4dc45ad4af03270add2686fe16688d3
---
BioMCP's JATS converter can display supplement filenames that the asset resolver cannot retrieve. On current `main`, article text for PMID 20516115 names two supplements (`Supplementary_Methods__Figures__Tables.pdf` and `Supplementary_Tables.xls`), while `get article 20516115 assets --json` returns no handles. Prior work made the empty/failure outcome more honest and added PMC OA, Europe PMC ZIP, and Figshare sibling retrieval, but the resolver still ignores supplement links carried directly by JATS or PMC HTML.

Imported from March ticket 600. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/600-resolve-jats-and-pmc-html-linked-article-supplements-through-stable-asset-handles
