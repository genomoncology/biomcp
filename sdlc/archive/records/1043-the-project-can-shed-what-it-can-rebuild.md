---
base: 6f374672b8ba2f67b3f4519dfcd52bcf6ecee742
head: 20d30008699170ffa779537182046b2e113dbca8
---
Add `sdlc/scripts/clean` so preserved worktrees can discard only rebuildable
Cargo output. The script is contract-tested so cleanup preserves source, git
state, and other local evidence.
