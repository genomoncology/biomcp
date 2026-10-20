---
base: d359e131
head: 961e42b9
---

The canonical Python contract lane now uses four bounded pytest-xdist workers
with file-based distribution. `PYTEST_WORKERS=1` retains a one-worker
diagnostic path, and source contracts reject an unbounded automatic worker
count, missing file distribution, or a sequential default.

The pull-request contracts job now calls the same `make test-contracts` target
with its already-built release executable. This removes the duplicate
sequential pytest and documentation command sequence without changing the
collected tests, their assertions, or product behavior.

A prepared-binary probe passed 500 tests in 43.28s versus about 105s
sequentially. Two complete canonical four-worker runs then passed 502 tests in
44.26s and 44.47s; the warm lane, including artifact preparation, Python
environment synchronization, and strict documentation, took 47.09s. On exact
implementation commit `961e42b9`, 503 tests and strict documentation passed in
45.07s, with pytest itself taking 39.91s.
