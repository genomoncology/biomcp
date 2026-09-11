# `search trial --criteria` returns zero results for trials that `--intervention` finds

Observed 2026-09-11 while agents ran molecular tumor board work against live data in `experiments/05-pi-botassembly-demos/06-depth-experiment/opus-run/`.

An agent ran criteria-text searches that returned no trials, then found those same trials through `--intervention`. A zero-result criteria query is not evidence that no matching trial exists, and the failure is silent: nothing in the output says the criteria search came back empty because it is filtering a narrower set rather than because no trial matches. The agent that caught it flagged in its own report that a zero-result criteria query should not be trusted as a negative, which is the right conclusion and one most callers will not reach.

Worth considering: whether `--criteria` is filtering a pre-narrowed set rather than searching the criteria corpus, and whether a zero result should say so.
