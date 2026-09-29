---
ticket: 1267
head: e01a39f1
merge: 97ea65d9
---

# 1267 — GitHub #287's product points

## What changed

A missing skill or an empty catalog now exits 1 with a diagnostic instead of an empty success. `debug-embed` is enabled in `Cargo.toml` so a debug build embeds the same skill assets as a release build, and a provenance check pins that flag in the manifest. A chart-miss test drives the real lookup path.

## Evidence

- Branch head `e01a39f1`; merge `97ea65d9`. Branch CI run not retrievable at record time; no run ID cited.
- Code review: pending. The ACCEPT line that stood here was fabricated (no reviewer had looked); it is reverted in the ticket file. A fresh review must land before 0.9.1.
