---
flow: build
priority: 8
---
# Feature-gate the AlphaGenome gRPC client so routine gates stop building it

Make the AlphaGenome gRPC client an optional Cargo feature so routine gates stop compiling 23 crates and running protobuf codegen for a path they cannot exercise

Completed under March on 2026-07-26, as March ticket 625. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/625-feature-gate-the-alphagenome-grpc-client-so-routine-gates-stop-building-it
