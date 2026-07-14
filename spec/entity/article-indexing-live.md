# Live PubMed Article Indexing

This operator-run canary checks the real PubMed citation boundary that supplies
researcher and MeSH indexing metadata. It deliberately asserts only stable
response shape because PubMed can revise individual citation fields over time.

## Live PubMed indexing canary

The documented PMID should return available indexing with at least one author
and one MeSH heading. `--no-cache` ensures this verifies the current upstream
response rather than a previously cached citation.

```bash
../../tools/biomcp-ci --no-cache --json get article 22663011 indexing | mustmatch like '{"indexing":{"status":"available","authors":[{}],"mesh_headings":[{}]}}'
```

## Live Europe PMC supplementary asset canary
<!-- mustmatch-lint: skip -->

The reviewed open-access article should expose at least one supplementary asset
through Europe PMC. The follow-up uses the returned filename rather than pinning
a mutable live filename, count, size, timestamp, or hash.

```bash run id=live-europepmc-assets exit=0
../../tools/biomcp-ci --no-cache --json get article 38821914 assets
```

```json expect=live-europepmc-assets contains
{"provider":{"source":"Europe PMC"},"assets":[{}]}
```

```bash run id=live-europepmc-first-asset uses=live-europepmc-assets exit=0
../../tools/biomcp-ci --no-cache get article 38821914 asset "{{live-europepmc-assets.assets.0.filename}}" | wc -c
```

```text expect=live-europepmc-first-asset
/^[1-9][0-9]*$/
```
