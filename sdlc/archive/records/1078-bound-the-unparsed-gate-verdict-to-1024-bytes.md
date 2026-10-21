---
base: e9bd0b2a6270c42465b9661629e256508d613050
head: 04b31c3a338e3678bf9c1d90ed5f5eeafd28ad26
---

The fallback gate verdict now reserves the header newline and stays within
1,024 bytes. A lifecycle contract covers long non-TAP gate output so the full
verdict bound cannot regress.
