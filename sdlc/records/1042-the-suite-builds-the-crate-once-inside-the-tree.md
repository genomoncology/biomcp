---
base: 663982f42d24e4dc9bd95eb47d0552b94b5957da
head: 4f2a9f80f72cf9173f36de934f57ab84603a8d26
---

# The suite builds the crate once, inside the tree

The routine test lane now keeps Cargo, pytest, and documentation scratch paths
under the ignored worktree cache. The packaging compile reuses the worktree
target directory while still compiling the extracted shipped source.
