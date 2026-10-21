---
flow: quickfix
priority: 2
---
# Closed by operator correction: reconcile recovered records

Closed without a factory flight on 2026-08-10. Build and quickfix flows may
edit only their own completion record, so the original runnable ticket was
impossible: it asked a 0944 flight to change records 0158, 0160, and 0161.

An operator independently proved each ticket-owned branch patch against the
main-reachable landed commit, recorded explicit exclusions and normalized
patch hashes in the three records, and changed only their `head` plus recovery
notes. The corrected landed heads are:

- 0158: `fb56bd624c0a984ba7c76839048859556e4e5190`;
- 0160: `f68a2589043cd3b97cf825b60f524548751d21b7`; and
- 0161: `7bca6b8163716d23b70937f4947c8f5f1e6a2033`.

All objects exist; each recorded base is the landed head's parent; each landed
head is an ancestor of main. No product history was rewritten. Do not run 0944.
