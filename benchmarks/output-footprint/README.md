# Deterministic output-footprint benchmark

This Tier 1 benchmark measures the agent-facing output of a fixed BioMCP corpus:

- compact and `--full` article search (the headline comparison)
- variant search
- gene get with an explicit section
- trial search

The runner starts an isolated loopback replay server and reuses committed provider
payloads from `testdata/sources/`. It does not call live services. Output is measured
as UTF-8 bytes and with tiktoken's `cl100k_base` tokenizer. The tokenizer package is
pinned by `uv.lock`.

Run it from the repository root:

```bash
make output-footprint
```

The command emits a JSON report. It exits non-zero when any compact surface exceeds
its pinned byte ceiling in `run.py`; the compact-versus-full article delta remains the
headline figure. CI runs this target so output growth cannot silently pass. Update a
ceiling only after reviewing an intentional agent-context trade-off and recording fresh
benchmark figures.
