---
base: 9cfd5e2957258f88051f8449df0b991bb5563879
head: 0e65c22c3c993e102f2249f1f64744173c52080c
---

BioMCP now sends PharmGKB annotation and health requests to the replacement
ClinPGx host. It omits rejected upstream pagination properties and applies the
requested bounded page after mapping valid annotation rows.

The health table names the host move so a future endpoint regression is clear.
