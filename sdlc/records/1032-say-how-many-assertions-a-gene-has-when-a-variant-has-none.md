---
base: 5965c6b2cc7f03347b55a0a61f490e9f7186b1a3
head: d6ebf57ece4fef9f1ff4f8dfbf9e312045a2a703
---

# Say how many assertions a gene has when a variant has none

Empty human-readable ERepo variant responses now name the known gene and the
number of assertions the repository holds for it. A zero count remains distinct
from unavailable lookup data, which preserves the successful empty response.
