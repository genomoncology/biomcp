---
base: 47eb946dbfaaad3fdba4693d66ecff68254c3c47
head: 5e5ef4ebff420fbcd0cd2dd69494effe4c15b51e
---

Variant search now reports why provider filters returned no rows. It retries
known gene aliases, suggests bounded protein positions, and distinguishes a
true empty intersection so callers can interpret zero results safely.
