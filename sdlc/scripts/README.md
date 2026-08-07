# sdlc/scripts/

Copied into a repository at onboarding. These scripts are how *this* project
does worktrees, gates and merges — edit them freely; nothing above reads them
except by name.

| Script      | Contract                                             |
| ----------- | ---------------------------------------------------- |
| `prepare`   | verify origin/main is green, claim the branch on origin, make a tree, print its path on stdout. Exit 3 claimed elsewhere, 4 environment refuses, else fault (queue ADR 0009) |
| `integrate` | tear down after a landed run. The landing *and* the record both happen inside the flow (sdlc ADR 0008), so nothing is written here |
| `discard`   | tear down after a run that did not land. Only `blocked` keeps its tree and branch (sdlc ADR 0005) |
| `lint` `test` `spec` | the gate ladder. Cheapest first             |
| `doctor`    | report on this project's SDLC health. Never deletes   |

Branch from `origin/main`, never local `main` — a local main may be stale or
dirty and differs between machines. This is deliberate; do not simplify it.
