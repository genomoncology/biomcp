---
flow: build
priority: 6
---

# A stopped trial never says why it stopped

`biomcp get trial NCT03515785 all` reports `Status: WITHDRAWN` and never says why. Verified against the repository build `biomcp 0.9.0-dev.6` (`./target/debug/biomcp`) on 2026-09-01.

The reason is already in the payload BioMCP fetches:

```
"whyStopped": "Revisiting the availability of patients with Ph+ ALL that would meet
               the in-/exclusion criteria of the study led to the decision not to
               move forward."
```

The distinction the field carries is the one a reader needs. A trial that stopped because the drug failed and a trial that stopped because the sponsor could not recruit are different evidence about that drug, and the status word alone reads the same for both.

This is not rare. Three of the ten results from `biomcp gene trials RB1` carried a WITHDRAWN, SUSPENDED or UNKNOWN status.

## Required behavior

A trial whose status indicates it stopped shows the reason the registry gives, wherever that status is shown.

A trial that stopped and gives no reason is distinguishable from one whose reason was not requested.

## Done, observably

- `get trial NCT03515785` shows the withdrawal reason alongside the status.
- Terminated, withdrawn and suspended trials show their reason in markdown and in JSON.
- A trial list that shows a stopped status makes the reason reachable without the reader having to guess which section holds it.

## Boundary

This ticket does not change trial search, ranking, or which sections exist. Trials with an ordinary status keep their current output.
