---
base: 4c429b8f70ded07e26a7518def154680b3c8e9aa
head: 14109bb78f13306d7186408bc04d63694b92571f
---

Made the contradictory variant-filter contract reliable under parallel load.
The test bypasses shared cache state and verifies that invalid filters never
contact its local MyVariant listener while retaining the JSON error contract.
