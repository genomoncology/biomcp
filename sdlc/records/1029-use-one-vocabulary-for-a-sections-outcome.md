---
base: 7d534127d50039bb85b737a0860e004e4592ff87
head: a5e38a81e18e25049c89bdb12ea2ef045d14c54a
---

# Use one vocabulary for a section's outcome

Population status now emits the shared `data`, `empty`, `unavailable`, and
`inapplicable` outcome vocabulary. Legacy status spellings remain accepted on
deserialization, while the documented contract and regression tests prevent a
separate population vocabulary from returning.
