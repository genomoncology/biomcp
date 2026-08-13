---
flow: build
priority: 7
---
# Validate exact GWAS p-value exponents before comparison

Exact GWAS p-values preserve the provider's mantissa and signed 32-bit
exponent, then compare scientific values by adding the exponent to the digit
count. An extreme malformed exponent overflows that unchecked addition in
debug builds. Release builds wrap the value and can silently misorder or
misfilter associations.

Reject provider p-value parts outside a scientifically credible, explicitly
documented range before they become a `GwasPValue`, and make comparison
arithmetic safe even for malformed in-memory values. A rejected exact pair may
fall back to a valid positive numeric field; otherwise the association has no
usable p-value. Never publish a fabricated value.

## Done when

- Minimum and maximum signed exponents cannot panic or wrap during parsing,
  ordering, or threshold filtering in debug or release builds.
- Valid underflowed values such as `3e-1315` retain their exact current JSON,
  Markdown, ordering, and filtering behavior.
- Malformed exact parts follow the documented fallback-or-absence rule and do
  not become a plausible but false p-value.
- Receipt-backed normal provider fixtures remain unchanged unless their bytes
  themselves demonstrate invalid data.

## Authorized test changes

Design may restate GWAS parsing assertions in `src/sources/gwas/tests/parsing.rs`,
exact comparison and filtering assertions in
`src/entities/variant/gwas/tests.rs`, and rendered p-value assertions in
`src/cli/gwas/tests.rs` and `src/render/markdown/variant/tests.rs`.
