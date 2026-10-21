---
base: e8eebdec405f2c9b5dcd66668ed8f5ec4a4eae50
head: 4b94b161bd349efe8dfe2e8fd0c19f5ad91579f6
---

CSpec now renders manifests and selected criteria as Markdown by default while
preserving JSON with `--json`. ERepo detail Markdown includes its narrative and
source URL, and batch input renders Markdown summaries without requiring JSON.
This makes each accepted flag observable instead of silently ignored.
