# 2087 — Make error messages name the real failure

Status: OPEN.

Milestone: 0.9.3

Reviews: revision eac19b0, accept

## Outcome

`get trial NCT99999999` reports the trial as not found, `get article 999999999999` reports no article with that PMID, and `get variant "BRAF Z999Q"` refuses the invalid residue with exit 2.

## Evidence

Split from 2068 on 2026-10-14; found by the post-release install and product test of v0.9.2 and rechecked against current main.

- `get trial NCT99999999` says "Retry the remote source" for a trial that does not exist. `get_detail` wraps the correct NotFound in `SourceContext::retry`, which renders that text; `tests/test_ctgov_trial_search_detail_reuse.py:454` asserts the wrong text.
- `get article 999999999999` says "Unsupported identifier format… PMID (digits only)" for an all-digit input, because `parse_pmid` parses into a `u32` and anything above 4294967295 overflows into the unsupported-format branch.
- `get variant "BRAF Z999Q"` reports not found and suggests a search with Z. `gene_protein_re` accepts any capital letter; `amino_acid_one_letter` would reject Z but is not called on that path.
- PM decision 2026-10-14: an all-digit identifier is always treated as a PMID and parsed as `u64`; a PMID with no record is "not found". A protein change with a letter that is not one of the 20 amino acids, `X` or `*` is an invalid argument (exit 2) naming the bad letter, before any lookup.

- Starts from: `get_detail` (`src/sources/clinicaltrials.rs:446-459`) and the retry rendering in `src/error.rs` (line 251 and 770-786); `parse_pmid` (`src/entities/article/detail.rs:31-40`) and the unsupported-format message (`src/entities/article/detail.rs:606`); `gene_protein_re` (`src/entities/variant/resolution.rs:257-260`) and `amino_acid_one_letter` (`src/entities/variant/resolution.rs:525-550`); the not-found suggestion in `src/entities/variant/get.rs:250-255`.
- Keeps: the retry message for real transport failures; every valid identifier's resolution.
- Changes: claims `src/sources/clinicaltrials.rs` `tests/test_ctgov_trial_search_detail_reuse.py` `src/entities/article/detail.rs` `src/entities/article/detail/tests.rs` `src/entities/variant/resolution.rs` `src/entities/variant/resolution/tests.rs` `CHANGELOG.md`.
  - `get_detail` passes a NotFound through without the retry context.
  - `parse_pmid` returns `u64`.
  - The protein-change path checks each residue letter with `amino_acid_one_letter` and returns an invalid-argument error naming the letter.
  - Fix the wrong assertion at `tests/test_ctgov_trial_search_detail_reuse.py:454`.
  - One Fixes changelog bullet.
- Proof: recorded or unit tests run through the CLI for NCT99999999 (not found, with the trial-search suggestion), 999999999999 (not found as a PMID), and BRAF Z999Q (invalid residue, exit 2). Each fails on current main. Routine suite.
- Defers: nothing.
- Order: runs alongside the build tickets and 2066; it shares no claimed file with 2065, 2066, 2067, 2084 or 2085.
