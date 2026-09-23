# FDA label warning follow-ups after ticket 1232

Filed 2026-09-23 from a review of ticket 1232 (merge b773c893). The boxed and legacy warnings now reach both label paths in Markdown and JSON, and the tests fail if the boxed warning is dropped.

## Should-fix

- The safety-path warnings are never truncated. `extract_label_warnings_text` (`src/entities/drug/label.rs:396-403`) has no cap, while the boxed warning beside it is capped at `:410`. Legacy `warnings` sections now flow here and can be long. Apply the same cap and add a test.
- A truncated warning gives no way to the full text. The note at `label.rs:40` says only "(truncated, N chars total)". The DailyMed link appears only when `drug.label` is present (`src/render/markdown/evidence.rs:352`). Put the DailyMed link from `label_set_id` in the truncation note.

## Minor

- Headings differ: "Boxed Warning" (`templates/drug.md.j2:23`) and "FDA boxed warning" (`src/render/markdown/drug_regulatory.rs:343`). Legacy text renders under the modern "Warnings and Precautions" name (`drug.md.j2:32`). When both sections are requested the boxed warning prints twice. Use one heading, a neutral "Warnings" heading for legacy text, and print the boxed warning once.
- No fixture has both `warnings_and_cautions` and `warnings`, so the precedence at `label.rs:363-364` is unpinned. No JSON test asserts `boxed_warning` in the output. No test covers a multi-string or truncated boxed warning.
- `docs/user-guide/drug.md:89-95` and `docs/sources/openfda.md:24` do not mention boxed warnings.
- `src/render/markdown/drug/tests.rs` was added to the size inventory at 1084 lines to fit two tests (`tools/rust-source-size-inventory.json:285-295`). Move them to a new test file and drop the entry. The `src/render/provenance.rs` reason text describes logic that lives in `get.rs:1062-1070`; correct it.
