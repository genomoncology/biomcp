---
base: e7fb9e6ec3ea578a64ba66ac13314672a2f71ac2
head: c946f8419942827dd937a44649281718a06f7f7f
---
`get article <pmid> --json` deliberately reduces author lists longer than four to first and last author. PMIDs 35637217, 37449980, and 38821914 return 2 authors although PubTator/PubMed carry 16, 28, and 18. The field looks complete, so middle-author attribution fails silently. `article batch` omits authors entirely even though its underlying article objects already have them.

Imported from March ticket 512. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/512-return-complete-article-authors-without-silent-truncation
