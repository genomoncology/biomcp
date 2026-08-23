# Worktree pre-commit hook runs another checkout's script

A linked worktree's Git pre-commit hook points at the primary checkout by
absolute path. It can run an older `scripts/pre-commit` than the worktree's
staged changes, so a documentation-only commit prints the MkDocs Material
banner even though the worktree's current script passes
`NO_MKDOCS_2_WARNING=1`.

Use a worktree-local or revision-correct hook path so commit-time checks match
the staged checkout.
