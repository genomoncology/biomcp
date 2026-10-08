## Lanes

Drafted by `pm init` from the linked worktrees. Holds cells stay blank for review: write backtick globs or `free`.

| Lane | Worktree | Holds |
| --- | --- | --- |
| A | (free) | lane A idle since 1294 landed and its worktree went away; `src/transform/article/*` |
| B | (free) | lane B idle; `src/cli/variant/*` `src/entities/variant/*` |
| C | (free) | lane C idle, queue empty for it; `src/entities/disease/*` `src/entities/article/*` |

The 0.9 queue's open work runs in per-ticket worktrees outside these lanes:
`worktrees/biomcp-1291fix` (1291), `worktrees/biomcp-2016` (2016),
`worktrees/biomcp-2020fix` (2020), `worktrees/biomcp-2021fix` (2021),
`worktrees/biomcp-2022fix` (2022), and `worktrees/biomcp-2029` (2029).
The third review's tickets 2030, 2031, and 2032 run in
`worktrees/biomcp-2030`, `worktrees/biomcp-2031`, and
`worktrees/biomcp-2032`. Ticket 1305's changelog lane holds no
worktree. Tickets 2023, 2024, 1300, and 2017 landed on main.
