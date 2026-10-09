# 2031 — Drug name lookup never swaps the drug

Status: OPEN.

Milestone: 0.9.2

## Outcome

`biomcp get drug <name>` returns the named drug or says it found no match. It never returns a different drug's card or label without saying so.

## Evidence

Filed 2026-10-08 from the third review of the work since v0.9.1 (ticket 2033, finding 2). Ticket 1300 did not cause this; its diff does not touch name resolution.

- Live on main `7d17fd099`: `get drug terfenadine` returns the fexofenadine hydrochloride card and fexofenadine's label.
- `get drug mannitol` returns a card named "analgesic" with a minor-burns label. Edetate disodium resolves to a urea foot cream, and ferric oxide to calamine.
- The 1300 review called terfenadine "honestly empty". That does not hold live.

- Starts from: the MyChem name resolution `get drug` uses, and ticket 1300's identity guard for labels.
- Keeps: correct resolution for drugs whose name or synonym matches, such as osimertinib, TAGRISSO, lazertinib, amivantamab and mobocertinib.
- Changes: accept a MyChem hit only when the query matches its name, a brand name or a listed synonym; otherwise report no match, or list the candidates with a note.
- Proof: terfenadine and mannitol return no wrong card, and a test pins each against recorded captures; the five correct resolutions above still pass.
- Defers: nothing.

## Head review (fourth-review fix round)

- Code re-review (full head through c0dc28edf, covering the discover
  rescue and the brand fix the prior ACCEPT missed): ACCEPT 2026-10-08.
  Verified: the TAGRISSO resolution through MyChem's brand fields
  against the real capture (get and search agree); the rescue-miss
  keeps the sparse card; the four pinned cases hold; the keep-list
  resolves live; the parent-acid nuance and the pooling residuals
  recorded. Two report-only P2s: no automated pin on the rescue-miss
  path (deferred to the merge-pooling lane the ticket names), and the
  capture-header minimization sentence under-describes the dropped
  non-name field blocks.
