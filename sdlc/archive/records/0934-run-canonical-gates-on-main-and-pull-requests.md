---
base: 8014e0927199a6abc8f46cb6501d841a241bc920
head: d64845c5999641715a9e87c3399950ca66b256bf
---

Pull requests and every push to main now call the repository's canonical lint,
test, specification, and shipped-feature gates instead of maintaining weaker
workflow-local command copies. Source contracts pin the workflow triggers and
gate delegation.
