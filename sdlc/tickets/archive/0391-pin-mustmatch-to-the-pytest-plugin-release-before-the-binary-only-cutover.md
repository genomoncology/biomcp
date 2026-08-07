---
flow: quickfix
priority: 5
---
# Pin mustmatch to the pytest-plugin release before the binary-only cutover

mustmatch is cutting over to a single static Rust binary (mustmatch team ticket 11, in flight): the Python CLI, PyO3 layer, and **the pytest plugin** are deleted and it ships as a new binary-only version. biomcp is the only repo that consumes the pytest plugin (`pytest spec/ --mustmatch-lang bash …`), and its `pyproject.toml` floats `mustmatch>=0.0.4` with no upper bound. When biomcp next syncs Python deps (`make sync-python-dev`, called by `make spec`), it would pull the binary-only version, the pytest plugin would be gone, and `make spec` would break. Pinning to the last pytest-plugin release decouples biomcp from ticket 11's release so it keeps working until its own spec-runner migration lands. This is the decoupling the mustmatch cutover's release-coordination decision depends on; it must be in place before ticket 11 publishes.

Completed under March on 2026-06-04, as March ticket 391. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/391-pin-mustmatch-to-the-pytest-plugin-release-before-the-binary-only-cutover
