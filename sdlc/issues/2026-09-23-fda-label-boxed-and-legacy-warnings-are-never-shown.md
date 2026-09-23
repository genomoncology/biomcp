# FDA label boxed and legacy warnings are never shown

Filed 2026-09-23 from an independent review of `v0.9.0..f2549676`. Blocks 0.9.1.

## Symptom

`src/entities/drug/label.rs:362` and `:396` read only `warnings_and_cautions`. No code in `src/` or `templates/` reads `boxed_warning`. Older-format labels keep their warnings in `warnings`.

`biomcp get drug vincristine safety --region us` prints "FDA label warnings: No data found". The label fetch succeeded, so the safety summary lists the label as a source with nothing to report. A boxed warning never reaches the reader for any drug.

## Reproduction

```
grep -rn boxed_warning src templates        # no match
biomcp get drug vincristine safety --region us
```

## Fix

Read `boxed_warning`, then `warnings_and_cautions`, then `warnings`. Render the boxed warning in its own block ahead of the others. Add fixtures for a current-format label with a boxed warning and an older-format label with only `warnings`.
