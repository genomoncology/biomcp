---
base: 2cf28f4f
head: 65e574eb
---

The installer now stages verified bytes in the destination directory, smokes
and syncs them, records recoverable pending state, commits with one rename, and
finalizes ownership. It refuses links and cleans owned stages. All repository
gates passed.
