# 2048 — Make pm review records pass the contract test and close landed tickets

Status: OPEN.

Milestone: 0.9.2

## Outcome

`pm ticket review` records a verdict that the repository contracts read as valid, so no ticket record is edited by script. pm shows every 0.9.2 verdict. Every landed 0.9.2 ticket has a verdict on its landed head and is closed through pm. Landed branches and worktrees are gone.

## Evidence

Filed 2026-10-10 from the independent records audit of main at `c6d972d47` and `origin/tickets/landcheck-C` at `ac4c6739a`.

- `pm ticket review` writes `Reviews: revision <sha>, <verdict>` (`src/commands/ticket.ts:110` in the pm repo), and pm reads verdicts only from that line. `tests/test_sdlc_review_status_contract.py` treats any `Reviews:` line as a malformed status line. Every pm review commit therefore fails repository-contracts: dd5a5cf9b, aff34ee66, 8e20f5b25 and 763338347 on landcheck-C, plus earlier ones on landcheck-A and landcheck-B.
- The team then deletes pm's line and writes prose by script (commits `aa0648e2d`, `d0d6da72a`, `ac4c6739a`). Main and landcheck-C now carry no `Reviews:` line at all, so `pm item` reports no verdict for 2038 and 2042 to 2045, and the rule against editing records by script is broken on every review.
- Verdicts name heads other than what landed. 2036 has none in its ticket. 2037's covers a records-only note (`730f4ae19`). 2043's names `41acc461f`, and `bf8861a2a` then changed three source files. The 2044 and 2045 verdict names `21630b5eb`; merge `321969db0` resolved a conflict by hand, and four commits followed on main (`2e3c1523b`, `8265b7bbc`, `2f2bad601`, `c6d972d47`). No CI run exists for `2285a3bac`, `619dc2a1e` or `321969db0` alone.
- 2036, 2037, 2042, 2043, 2044 and 2045 are landed and still say `Status: OPEN`. `pm daily` names `1b9d15609` as the 2042 landing; the merge is `2285a3bac`.
- Remote branches already in main remain: hold/parent-records, tickets/2022-record-fix, tickets/2037-review-note, tickets/2038-rc-contributing-record, tickets/2038-rc-docs, tickets/2038-rc-drug-label-variant-hint, tickets/2038-rc-size-inventory, tickets/2038-records, tickets/2042-fix, tickets/2043-fix, tickets/2044-fix, tickets/2044-records, tickets/landcheck-A and tickets/landcheck-B. Eleven `biomcp-*` worktrees remain on the dev box.
- pm's 2026-10-07 asks (lanes building on the dev box, issues retire into tickets, lighter debug info) are unanswered. The team's own reply and a message to pm sit in the biomcp inbox with no header, so pm reads them as malformed.
- Recent record sections are hard-wrapped: the 2036 and 2042 build outcomes, the 2038 sections, and every hand-written verdict section. 2039, 2040 and 2045 say "Filed 2026-10-10" though git shows 2026-10-09. 1300 has no `Milestone:` line.

- Starts from: main `c6d972d47`.
- Keeps: the review grammar for hand-written records; the one-verdict-per-line rule.
- Changes: make the contract test treat pm's exact `Reviews: revision <sha>, <verdict>` line as a valid verdict; restore pm's lines through `pm ticket review`; record a fresh verdict on each landed head, by a reviewer who ran the gates and says which; close each landed ticket with `pm ticket land`; remove landed branches and worktrees; answer pm's asks with headed messages; unwrap the hard-wrapped sections through a reviewed records commit.
- Proof: a pm review commit passes repository-contracts on CI; `pm item` shows a verdict for every 0.9.2 ticket; `pm daily` lists no landed 0.9.2 ticket as open.
- Defers: nothing.
