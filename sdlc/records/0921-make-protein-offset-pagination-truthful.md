---
base: 9259bc2c4ab8379602f666322ebb82dbe4a9692b
head: dcbc1d94d6f5e65559e6e2de96946d7620cf3df8
---

Protein search now applies offset and limit to the filtered result set and
reports a next offset only when another row exists. `has_more` is derived from
provider evidence rather than assuming that a full page guarantees another
page.

Exact-end, short-page, and continuing-page cases pass in JSON and human output
through the local UniProt transport and the focused protein test suite.
