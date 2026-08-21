---
flow: build
priority: 7
opens: sdlc/scripts/clean
---
# The project can shed what it can rebuild

This repository tells the factory how to build and check itself through the
scripts in `sdlc/scripts/` — `install`, `lint`, `test`, `spec`. The lifecycle
scripts above them know git and know nothing about Rust, and ask down through
that ladder whenever they need something only this project can answer.

There is no way for them to ask this project to throw away build output. That
matters because a worktree kept for a human to look at keeps everything: on
2026-08-21 three worktrees of this repository held 14, 14, and 18 GB, almost
entirely `target/`, against 28K of files an agent actually wrote. One of those
attempts had already died with `No space left on device` while archiving test
binaries, costing the ticket an attempt on a condition that had nothing to do
with the work.

The behavior: `sdlc/scripts/clean` removes build output that this project can
rebuild, and leaves everything else alone.

Done, observably:

- Run in a worktree of this repository, it removes the cargo target directory
  and exits 0.
- It leaves the source tree, the git state, and anything an agent wrote
  untouched. A worktree that has been cleaned still builds, still tests, and
  still shows the same `git status` as before.
- Running it twice is the same as running it once. A worktree with nothing to
  shed is not an error.
- It says on stdout what it removed, or that there was nothing to remove.

Settled choices:

- **Only what a build regenerates.** The shared compile cache under
  `~/.cache/sccache` is not this script's business — it is deliberately sized
  in the dotfiles, shared across every worktree, self-evicting, and it is what
  makes discarding a target directory cheap in the first place. Nor does this
  script touch the Python virtual environment or anything under `.cache` that
  a run produced as evidence. If a doubt arises about whether something is
  rebuildable, it stays.
- **It never chooses its own moment.** The script removes when asked and
  makes no judgement about whether now is a good time; the caller decides
  that.

Consumers: `sdlc/project/failure` calls this script by name when it preserves
a blocked attempt, per sdlc ticket 0138. That caller treats a missing script
as ordinary and a failing one as non-fatal, so this script standing alone
changes nothing until that lands and propagates, and it can land first.

The behavior being replaced: none. This is a new script and no shipped
assertion describes it.
