---
base: 9d18fb8f
head: ea055a0c
---

The preventative Cargo source package boundary now excludes `architecture/`
and `sdlc/`, reducing inventory from 2,642 files to 1,254 while retaining
runtime assets, public docs, specs, skills, templates, sources, and the focused
package test. The region-alias assertion now reads shipped public docs instead
of private architecture history.

Cargo verifies the normalized package offline, then a safely extracted copy
outside the checkout compiles and runs one package-safe integration target
linked to the packaged library. Static checks reject private compile-time
includes. The boundary suite passed inside network isolation, canonical lint
passed, and independent review accepted the final package proof.
