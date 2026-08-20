---
base: a86a3c73e4ce88af085a9bb2e922a4e8f9dea69f
head: 7561cb6e33fba8d83e9c57519f6f834a933ab42a
---

# Stop printing zero counts for a source that was never reached

Markdown sections now place typed unavailable, degraded, and inapplicable
source states in the affected section before navigation. Successful empty
outcomes retain their trustworthy zero-result wording, while unavailable
sources no longer imply a zero result.
