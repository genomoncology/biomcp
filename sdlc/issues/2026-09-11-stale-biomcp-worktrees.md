# Stale biomcp worktrees sit idle after a cleanup pass

Observed 2026-09-11 while clearing idle git worktrees across the workspace. Six biomcp worktrees that had gone idle (no process using them, no file changed in the last 24 hours) were removed: `biomcp-1147-design-refresh` (ticket/1147-integration), `biomcp-1147-implement` (ticket/1147-implementation), `biomcp-1151-design-refresh` (ticket/1151-implementation), `biomcp-1182` (ticket/1182-case21), and `genomoncology/biomcp/1126` (ticket/1126). Their branches were kept; only the working-directory copies were removed. Two remain, both skipped in that pass because a process had them open.

## biomcp-1.0

Branch `biodata/biomcp-1.0`. 20 commits ahead of main, 0 files dirty. Directory size 119G, of which 117G is the Rust `target/` build cache under this worktree, not source content. No ticket file for a "biomcp-1.0" or "1.0" migration exists in `sdlc/tickets/`, `sdlc/tickets/archive/`, or `sdlc/records/` — this branch is not tracked by an open or archived ticket.

## biomcp-1187

Branch `ticket/1187-restore-green-release-baseline`. 0 commits ahead of main (the branch points at the same commit as main), 7 files dirty. Directory size 11G. No ticket file for 1187 exists in `sdlc/tickets/` or `sdlc/tickets/archive/`. The worktree is carrying only uncommitted local changes on top of main, with no ticket recording what they are for.

## What should happen

Both worktrees have no corresponding ticket to point back to. Someone should look at what each is doing, then either land the work as a ticket and finish it, or drop the uncommitted changes and remove the worktree. `biomcp-1.0`'s branch has real commits and should get a ticket if the migration is still wanted, or be closed out. `biomcp-1187`'s dirty state should be committed under a ticket or discarded.

Ian does not need to be asked before either action. Land or withdraw each, then remove the worktree.
