---
base: 8014e0927199a6abc8f46cb6501d841a241bc920
head: 915b36581c2a709a2487249ef52abefb0a884772
---

Unix-socket tests now create their socket endpoints under a deliberately short
temporary root rather than inheriting potentially overlong checkout paths.
The tests retain cleanup and isolation while no longer depending on the
caller's directory length.
