---
base: e1095164
head: 68f6b3cc
---

All 23 trial checks now run routinely against one shared CT.gov fixture and
the shared provider server. Fresh production-path captures cover condition
search, terminal and cursor pagination, mutation inclusion/exclusion and its
detail checks, Keytruda aliases, age count, eligibility/location detail, and
a minimized NCI melanoma response. Existing synthetic edge fixtures continue
to own contact, empty-result, alias, and shell-safety cases.

Repeated per-block CT.gov setup was removed, so the routine runner starts the
server once. Exact CT.gov and NCI requests are asserted from fixture logs.
The focused page passed all 23 blocks; 51 lifecycle, receipt, and registry
tests passed. No source lines were added against the 160-line ceiling.
