---
base: 2cf28f4f
head: 65e574eb
---

Release archives now use the exact 256 MiB ceiling at both HTTP boundaries,
while metadata and checksums keep the smaller shared limit. Local declared and
chunked fixtures cover exact, plus-one, and a verified archive larger than 8
MiB. All repository gates passed.
