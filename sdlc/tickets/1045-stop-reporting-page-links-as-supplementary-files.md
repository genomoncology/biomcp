---
flow: build
priority: 6
---
# Stop reporting page links as supplementary files

The article asset manifest lists ordinary hyperlinks from the PMC page as named supplementary files that could not be retrieved. A caller reading the manifest sees a list of failures where there is actually nothing to fetch.

## What it looks like

`biomcp --json get article PMC11857949 assets` returns zero assets and a coverage list whose every entry is a link from the article page:

```
unsupported_origin | 'F1'          | 'Open in a new tab'
unsupported_origin | 'F7'          | 'Open in a new tab'
unsupported_origin | 'NCT01072175' | 'NCT01072175'
unsupported_origin | 'NCT01336634' | 'NCT01336634'
```

Those are figure-viewer anchors and ClinicalTrials.gov registrations. That article has no supplementary files at all. The same pattern puts license URLs into the list as files named `4.0` or `1.0`, and article DOIs as files named after the DOI suffix, on `PMC7857465` and `PMC8571879` among others.

Nothing is fetched from those links — the URL policy refuses them, which is why the outcome is `unsupported_origin`. The defect is that they were treated as supplement candidates at all, and are then reported to the caller as named files with a retrieval failure.

## Why it matters

Coverage is how a caller distinguishes "this article has no supplements" from "this article has supplements we could not reach". Filling it with page furniture destroys that distinction. An agent reading this manifest has to conclude the source is broken, when the correct answer is that there is nothing there.

## Where it comes from

`extract_pmc_supplement_links` in `src/transform/article/html.rs:71` keeps an anchor when it carries a `data-ga-action` containing `suppl`, or when any ancestor's class or id tokenizes to one of:

```rust
"sm" | "supp" | "supplement" | "supplementary"
```

`sm` is a common layout and sizing token in PMC's markup, so it matches containers with no relationship to supplementary material. Separately, nothing requires the link target to look like a retrievable file, so a link to another page or an external registry becomes a candidate with a filename derived from the last path segment.

## The hard choice to settle

Decide whether to narrow the ancestor tokens, to require the link target to look like a file, or to do both. Narrowing tokens alone still admits a genuine supplement container that links out to a registry; a file-shape test alone still admits stray files from unrelated containers. Whichever is chosen must not lose a real supplement that is currently found — `PMC3040717` and `PMC3549296` both name real supplements through this route, and both must still be named after the change even though PMC currently answers for them with a proof-of-work challenge.

## Done when

- `PMC11857949` reports no supplementary-file coverage entries, because it has none.
- The figure-viewer anchors, ClinicalTrials.gov links, license URLs, and bare article DOIs listed above no longer appear as named files in any manifest.
- `PMC3040717` still names `NIHMS265402-supplement-Supplementary_Methods__Figures__Tables.pdf` and `NIHMS265402-supplement-Supplementary_Tables.xls`, with their current `pmc_proof_of_work` outcome unchanged.
- `PMC7857465` still names and retrieves the supplement and figure assets it returns today.
- The proof runs from stored HTML, not a live call.

## Existing tests that pin this

`src/transform/article/html.rs` owns the extraction and its tests. Restatement is authorized in that file only, and only for tests that assert which anchors become supplement candidates. Name each one that is changed in the design commit message.

No test outside `src/transform/article/html.rs` is authorized. In particular, do not weaken the asset manifest and coverage assertions in `src/entities/article/assets.rs`; if one of those fails, the change has altered a real supplement and is wrong.
