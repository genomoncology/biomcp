---
base: 33e578223d776ad987ed115a1036953408fc36a2
head: 436c8262fd22b49676e3542e9ee4113b6fcc5355
---
Make the AlphaGenome gRPC client an optional Cargo feature so routine gates stop compiling 23 crates and running protobuf codegen for a path they cannot exercise

Imported from March ticket 625. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/625-feature-gate-the-alphagenome-grpc-client-so-routine-gates-stop-building-it
