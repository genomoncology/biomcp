# Keep drug Markdown regulatory regions isolated

File/line: `src/render/markdown/drug/tests.rs:363-381`
Severity: should-fix

Despite its name, the regional rendering test searches the complete document.
US BLA/OpenFDA facts and EU EMA facts can be moved, merged, or flattened across
regional headings while it passes. Slice the Markdown at regional headings and
assert representative rows and values only in their jurisdiction's subsection,
including cross-region absence checks.
