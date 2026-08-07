---
base: 068bcbdb57c820c96da612b459d05583fb6657f0
head: 098fffac6028904d04ef490033cff7cb1817bca9
---
`search article -a/--author` is documented by clap as “Filter by author name,” but the default federated route sends the name as a real author-field query to Europe PMC/PubMed and as free text to PubTator/Semantic Scholar, then unions all rows. `Williams LS` therefore returns Williams syndrome and unrelated lexical matches ahead of real byline matches. The flag is also absent from `biomcp list article`. Backend query syntax placed in `-k` is provider-neutralized inconsistently instead of being honored or rejected.

Imported from March ticket 513. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/513-make-article-author-filtering-exact-across-the-selected-backends
