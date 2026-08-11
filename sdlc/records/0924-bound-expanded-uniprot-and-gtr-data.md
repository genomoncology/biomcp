---
base: be1af1770d209add74a019884ef7b823238501b4
head: b9c653f3f1d890f5fed5f833e4df5139443a9172
---

Bound UniProt payload expansion to 32 MiB, GTR test-version expansion to 512
MiB, and both GTR datasets to one million data rows. Production entry points
always use those pinned constants; decoder and parser seams accept small limits
for routine tests.

Exact-limit and limit-plus-one tests use small local fixtures. Instrumented
readers prove byte rejection after limit plus one and row rejection at the
first excess row. A valid but over-budget GTR refresh is rejected before either
file is written, leaving the prior complete bundle unchanged. All 16 GTR and
18 UniProt tests pass. The implementation added 145 net source lines against
the ticket's 150-line ceiling.
