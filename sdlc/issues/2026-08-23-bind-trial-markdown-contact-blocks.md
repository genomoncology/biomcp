# Bind trial Markdown contacts to their blocks

File/line: `src/render/markdown/trial/tests.rs:225-242`; `spec/entity/trial.md:154-164`
Severity: should-fix

The test and executable spec search globally for central contact, eligibility,
and site-contact values. They pass when a site email moves outside its location
row or central and eligibility details move to another section. Assert the
Central Contact and Eligibility blocks and the complete Locations row within
section boundaries in both test and spec.
