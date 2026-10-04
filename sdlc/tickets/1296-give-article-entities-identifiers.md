# 1296 — give article entities identifiers

Proposed 2026-10-03 by the BioMCP owner for the next 0.9 release.

Status: OPEN.

## Outcome

`article entities` carries each annotation's identifier and namespace, keeps same-text different-identifier annotations separate, and prints a `get` command that opens the record. An agent can follow an entity from an article to its gene, disease or variant record.

## Evidence

- Starts from: The owner ran `biomcp article entities 30738221 -j` on 0.9.1. `extract_annotations` (`src/transform/article/annotations.rs:62-109`) reads only text and type and merges by lowercased text. Each annotation carries `text` and `count` only, for example `{"text":"KRAS","count":12}`. PubTator3, BioMCP's upstream for this command, returns an identifier and position for each annotation. Experiment 435 had to call PubTator3 directly to get them. BioMCP's ideal state says anything BioMCP prints as an identifier can be typed back in.
- Keeps: `text` and `count` stay, with the same meaning. Markdown output stays compact.
- Changes: See Change detail.
- Proof: A recorded PubTator3 response for PMID 30738221 and a spec page showing identifiers and the follow-up commands.
- Defers: Mapping MeSH disease identifiers to MONDO.

## Change detail

1. `extract_annotations` reads the identifier PubTator3 gives (for example NCBI Gene, MeSH, rsID or HGVS) and the identifier's namespace.
2. Annotations that share a mention text but differ in identifier stay separate.
3. JSON output carries passage positions behind an option or in `--full`. `PubTatorAnnotation` drops `locations` at parse time (`src/sources/pubtator.rs:376-380`), so this needs a parse change too.
4. Entity rows print a `get` command by identifier where BioMCP accepts that identifier, for example `get disease MESH:...`. Today they print text searches such as `search gene -q "KRAS"` (`src/render/markdown/related/article_support.rs:50-64`). Rows without an accepted identifier keep the text search.

## Review

- Design review: ACCEPT 2026-10-03 on the second pass, dispatch 9962598b (fresh SWE-2 researcher, read-only). The first pass accepted with notes; the revision recorded them.
