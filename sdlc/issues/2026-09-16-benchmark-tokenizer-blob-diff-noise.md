# Vendored tokenizer blob dominates every cycle diff

Observed 2026-09-16 during the v0.8.25..v0.9.0 growth audit.

`benchmarks/output-footprint/tokenizer-cache/9b5ad71b2ce5302211f9c61530b329a4922fc6a4` is the tiktoken `cl100k_base` cache file, committed in e24618e9 on 2026-07-21 so `run.py` can point `TIKTOKEN_CACHE_DIR` at it and keep a cold machine offline. The motive is sound. The file is 1.7 MB of one-token-per-line base64 across 100,256 lines. It accounted for 99.6% of this cycle's 100,604 inserted lines under `benchmarks/` and 27% of all insertions in the release. Any future re-vendor of that asset repeats the same distortion.

The working-tree cost is modest and the pack already compresses the blob well. The real cost is diff noise: `git diff --stat` and numstat readings of a cycle report the tokenizer before they report the product.

Worth considering: mark the file `-diff` in `.gitattributes` so git reports it as binary. That is a one-line change and removes the noise while keeping the offline guarantee. LFS or a documented one-time download are heavier alternatives.
