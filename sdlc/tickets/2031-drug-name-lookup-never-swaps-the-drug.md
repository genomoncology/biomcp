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
