---
base: cf22293c
head: 17ac88cc
---

Made the GenCC cancellation settle wait for generation temporaries,
from the CI failure observed during the 1245 merge (unrelated to that
merge's content).

The settle helper's poll loop now checks both cleanliness predicates
— no `.raw-` temporaries at the root and no `.tmp-` temporaries in
`generations/` — on every iteration before attempting the store, so a
lingering temp keeps the loop waiting on the shared deadline instead of
panicking; the etag verification runs only once everything is clean.
The old code checked the `.tmp-` condition once, immediately, after
settle, and the aborted publication's temp cleanup runs detached in a
blocking thread that can lag the refresh lock's release under load —
the exact single-shot shape the 1239 review's waiting rule names. The
change is line-neutral (the file stays at the ratchet's 1000-line
threshold) by folding the comment and deleting the now-redundant
post-settle re-read.

Evidence: combined review REJECT once (the first version was code
motion — the check moved inside the loop but still asserted rather
than polled) then ACCEPT after the rework, with the reviewer verifying
the poll order, the prefix and directory against the production
temp-creation paths, the second caller's safety, and falsifiability;
yellow gate at 17ac88cc — lint, test, and spec OK (one earlier cycle
caught a formatter shape; one run hit the unrelated disease-survival
reap flake, filed as ticket 1248, which passed three focused runs).

Residuals: none specific; ticket 1248 tracks the flake it surfaced.
