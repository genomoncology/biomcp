# Article full text lands in a hash-named cache file; no way to build a local library

`biomcp get article 41152957 fulltext` (v0.8.25) fetches the full text and reports:

    Saved to: /home/ian/.cache/biomcp/downloads/2c88c031a6eeb5281638c57a4254bc77.txt

The content is excellent — clean markdown with the whole article body. But the file name is a hash, the directory is a managed cache subject to `biomcp cache clean`/`clear`, and nothing maps the hash back to the PMID without re-running the command. A user who wants a searchable local collection of papers has to copy files out by hand and invent their own naming.

The wanted feature is a knowledge-base builder: let a fetch land in a directory the user owns, under a name a person and an agent can find later.

Sketch (smallest useful version first):

- `biomcp get article <id> fulltext --out DIR` writes `DIR/<pmid>-<slug>.md` with a small frontmatter block (pmid, pmcid, doi, title, journal, date, retrieved-at, source rung). Same flag on `asset <key>` for raw bytes.
- Nothing else changes: stdout behavior stays as-is when `--out` is absent; the cache keeps doing its job.

Later, if the small version earns it: a `library` concept (default directory via env/config, `biomcp library list/search`), and batch fetch into it.

Context: found 2026-08-27 while pulling PMID 41152957 (Wu G et al., ultra-rare variants across MNDs) for a reproduction experiment (`experiments/186-gangwu-biomcp-reproduction/`). The rolodex tool has the same gap from the other side — it saves papers to a user directory but only speaks arXiv, so biomedical journals are unreachable there. BioMCP with `--out` closes the loop for both.

Verified against current main on 2026-08-27: downloads resolve to the cache
root `downloads/` directory with hash filenames (`src/utils/download.rs:20`);
the cited hash file exists locally from ordinary use. Not a regression from the
0.9 line — present in 0.8.25 and current main alike.
