---
base: bf4817a382fa8784ecdbeced18651948a39fc5c0
head: e9a8eed922ad112ab8a07318841b7ceba7af4f4f
---
mustmatch is cutting over to a single static Rust binary (mustmatch team ticket 11, in flight): the Python CLI, PyO3 layer, and **the pytest plugin** are deleted and it ships as a new binary-only version. biomcp is the only repo that consumes the pytest plugin (`pytest spec/ --mustmatch-lang bash …`), and its `pyproject.toml` floats `mustmatch>=0.0.4` with no upper bound. When biomcp next syncs Python deps (`make sync-python-dev`, called by `make spec`), it would pull the binary-only version, the pytest plugin would be gone, and `make spec` would break. Pinning to the last pytest-plugin release decouples biomcp from ticket 11's release so it keeps working until its own spec-runner migration lands. This is the decoupling the mustmatch cutover's release-coordination decision depends on; it must be in place before ticket 11 publishes.

Imported from March ticket 391. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/391-pin-mustmatch-to-the-pytest-plugin-release-before-the-binary-only-cutover
