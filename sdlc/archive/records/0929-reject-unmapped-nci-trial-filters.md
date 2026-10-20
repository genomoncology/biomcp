---
base: f33103eab29c57ffbc25f5e97a2b3ef0ba920954
head: fcbbd2c50e195cbc27e375ebe25c21a3c7cf4a9d
---

NCI trial search now rejects study type, sponsor, update dates, and more than
one total biomarker/mutation/criteria value in the shared pre-client validator.
The CLI also rejects repeated or unquoted multi-token biomarker-like values
before dispatch, while CTGov behavior is unchanged.

An explicit 20-case table classifies the public NCI filter surface. Existing
request-plan observations cover every mapped CTS field, and the three
biomarker-like inputs each serialize exactly once. A fresh debug binary was run
against a counting loopback server for six rejected single and combination
cases; all failed and the server observed zero requests. All 85 trial-search,
37 trial CLI, 14 routine NCI source, 27 list-page, and 10 source-documentation
tests pass. The change added 98 net source lines against the 100-line ceiling.
