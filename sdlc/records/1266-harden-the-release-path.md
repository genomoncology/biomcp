---
base: 9eea4a86
head: 535747d9
---

Hardened the release path: the Homebrew tap waits for publish-release (the release is public before the formula lands); the wheel smoke asserts real asset content; a structural assertion requires --release --locked on every wheel build. Merged in the 535747d9 chain; CI run 36530006778 covers the
merge.
