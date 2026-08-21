---
flow: build
priority: 5
---
# Carry the JATS supplement caption into the asset manifest

A caller choosing which supplementary file to download sees only a filename. For most journals that filename is a publisher-internal string like `TBBE_A_1426496_SM7925.docx` or `12944_2021_1578_MOESM1_ESM.xlsx`, which says nothing about whether the file holds a cohort table, a protocol, or a supplementary figure. The article's own JATS XML usually says exactly that, and BioMCP already parses it and then drops it.

## What is there and what is taken

JATS `<supplementary-material>` elements carry a `<label>` ("Supplementary Table 3"), a `<caption>` describing the contents, an `xlink:href` to the file, and a media type. `extract_jats_supplement_links` in `src/transform/article/jats/supplements.rs` reads the href, the derived filename, the label, and the media type. It never reads the caption, and `ArticleSupplementLink` in `src/transform/article.rs:26` has no field to put one in.

The manifest type is ready for it. `ArticleAssetJats` in `src/entities/article/mod.rs:487` already has `label`, `caption`, and `source_id`, and the PMC OA package route already fills all three — `parse_jats_facts` reads the caption at `src/entities/article/assets.rs:2186`. Only the linked-supplement route is short: `src/entities/article/assets.rs:823` and `:911` both hardcode `caption: None` because the link facts never carried one.

So this ticket closes a gap between two routes that are supposed to describe the same thing the same way, rather than inventing a new field.

## Done when

- A supplement discovered through a JATS link reports its caption in the manifest's `jats.caption`, in the same place and the same shape the PMC OA package route already uses.
- A supplement whose JATS entry has no caption still appears, with the caption absent rather than empty.
- The label continues to be reported as it is today; the caption is additional, not a replacement.
- Caption text is bounded the way the rest of the JATS converter bounds text, and it is inline text with no markup passed through.
- The two routes agree: when the same file is found by both the package route and the linked route, merging them does not lose a caption that either one found. `merge` behavior for `jats` already exists at `src/entities/article/assets.rs:1134`.

## Scope

JATS only. `src/transform/article/html.rs:112` builds the same link type from PMC HTML, where there is no caption element to read; leave that route reporting no caption rather than guessing one from surrounding markup.

## Existing tests that pin this

`src/transform/article/jats/supplements.rs` contains `extracts_nested_and_standalone_supplement_media_with_typed_facts`, which constructs `ArticleSupplementLink` values and asserts filename, label, and media type on a two-link fixture. Restatement is authorized in that file, for that test by name, only to the extent needed to add a caption to the fixture and assert it. Do not weaken its existing filename, label, or media-type assertions, and do not change its deduplication expectation.

`src/entities/article/assets.rs` contains `build_manifest_hashes_binary_bytes_and_quotes_retrieval_commands`, whose JATS fixture already exercises `<caption>` on the package route and asserts the resulting label. That test is correct as written and is not authorized for restatement.

No other test file is authorized.

## Documentation

`architecture/functional/article-fulltext.md` currently says "Supplementary-material filenames and links remain display facts for the network-free converter" and describes the manifest's JATS facts. Update it to match whatever this ticket lands.
