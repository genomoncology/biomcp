---
base: 88422d01fccaa032ae33e9a236660b4bf3172375
head: b49ce732c100074ccd579810021a9a0b7713dc35
---

# Forbid hardcoded empty renderer commands

A default quality-ratchet audit now rejects hardcoded empty next commands in
Rust renderers. The guard preserves the shared JSON metadata command source and
explains the one-source policy when it fails.
