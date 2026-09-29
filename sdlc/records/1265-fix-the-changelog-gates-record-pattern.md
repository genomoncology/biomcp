---
base: 9eea4a86
head: 535747d9
---

Fixed the changelog gate's record pattern: ticket numbers are four digits under 2000, any slug following. The regression test drives record_tickets() end to end against a real git repo. Merged in the 535747d9 chain; CI run 36530006778 covers the
merge.
