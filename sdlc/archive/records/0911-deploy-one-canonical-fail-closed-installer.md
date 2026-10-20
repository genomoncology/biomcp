---
base: 2cf28f4f
head: 6989930c
---

Root `install.sh` is the sole authored installer, with a byte-identical deployed
copy. CI and release checks pin that identity, and a verifier compares public
bytes with the canonical file. All repository gates passed.
