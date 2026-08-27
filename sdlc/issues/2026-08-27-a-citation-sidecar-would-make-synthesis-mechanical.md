# A citation sidecar would make synthesis mechanical

Found in the botassembly knowledge-base run (2026-08-27,
experiments/188-shank3-knowledge-base): the synthesize stage had to scrape
evidence URLs and database attributions out of rendered markdown to cite
its knowledge-base files. It worked, but the mapping from claim to source
lives only in the prose layer — JSON output carries `_meta.evidence_urls`,
yet a consumer writing cited documents must reconcile markdown prose
against a separate JSON call.

The idea: a `--cite` flag (or a per-output sidecar file) that returns, for
one command's output, the structured citation set — upstream records with
labels and URLs, the section sources, and the identifiers used — so an
agent or script can render citations mechanically instead of pattern-
matching prose. Nothing about default output changes.

Not validated as a requirement by anyone outside our own experiments yet;
recorded for triage.
