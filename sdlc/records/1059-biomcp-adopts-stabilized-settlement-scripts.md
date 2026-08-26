---
base: ba0820b0282c22f2b4933058d6354ef46517c813
head: fe1c07931b5a994f6b2b3519caf8968cf1e45652
---

# BioMCP adopts stabilized settlement scripts

BioMCP now carries the complete canonical lifecycle set. Settlement preserves
retryable candidates, resolves withdrawal commands through `PATH`, and keeps
activation outcomes authoritative across teardown or interrupted landing.

Consumer contracts pin these settlement behaviors without changing BioMCP's
runtime surface.
