---
base: 8014e0927199a6abc8f46cb6501d841a241bc920
head: 3d019f8d666eb1b67d0ce485bf91b37167740ee6
---

The canonical lint gate discovers every tracked production shell file and
GitHub workflow, checks shell syntax, applies ShellCheck where appropriate,
and validates workflow structure with pinned tools. Fixture-repository tests
prove complete discovery without relying on the checkout layout.
