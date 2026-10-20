---
base: 71860f70db4df681ee49f6c8c5233543cc86b7e7
head: 74098bddce03234a07484dce8d9c680ad4ba96d2
---

# Make the documented FAERS count invocation work

A source-free adverse-event count now selects FAERS, matching the public
command reference. Explicit VAERS counts still fail with a source-specific
message, while the user guide documents both behaviors.
