# 1296 — give article entities identifiers

Proposed 2026-10-03 by the BioMCP owner for the next 0.9 release.

Status: complete.

## Build status

- Built on branch `tickets/1296-article-entity-identifiers`, sha `b57b8fc6e`, 2026-10-05.
- Code review: ACCEPT 2026-10-05, no blocking findings. One P2 (the hand-updated zero-coupling digest) closed by a local run of the ratchet suite, 35 passed.
- Identifier derivation prefers rsID, falls back to HGVS, never surfaces the tmVar composite; placeholders carry nothing. Same text with different identifiers stays separate. Positions serialize only behind `--full` (JSON). Get commands print only where the real parsers accept the form: disease MESH/OMIM via the crosswalk, variant rsID/HGVS via the classifier; gene and chemical rows keep text search.
- Spec page `spec/entity/article-entities.md` pins rows, compact-versus-full shapes, and markdown commands from the recorded 30738221 capture; 995 targeted tests, lint green, spec-pr article lane green.
- Deferred: MeSH-to-MONDO mapping (disease rows open via the crosswalk, no mapping needed for the outcome); get gene accepting NCBI Gene ids is separate-ticket territory.

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

- Design review: ACCEPT 2026-10-03 on the second pass, dispatch 9962598b (fresh researcher, read-only). The first pass accepted with notes; the revision recorded them.
