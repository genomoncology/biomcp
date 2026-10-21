---
base: 2cf28f4f
head: 65e574eb
---

Unix self-update now stages uniquely without following links, preserves modes,
smokes and syncs, revalidates ownership, records pending state, atomically
renames, and finalizes. Windows rejects before download. The checksum bypass is
gone. All repository gates passed.
