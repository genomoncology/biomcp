# Routine build profile

BioMCP keeps the AlphaGenome gRPC client in release artifacts, where an operator
can request a credentialed prediction. Routine gates build a smaller feature set
because their deterministic corpus cannot reach that optional client.

## Routine build omits AlphaGenome gRPC dependencies

The routine Cargo feature set is a successful local build configuration. Its
dependency graph must not bring in the AlphaGenome gRPC/protobuf stack or the
older web-stack versions that stack requires; otherwise a routine gate pays to
compile code it cannot exercise.

```bash
cargo tree --locked --no-default-features --edges normal,build | mustmatch not like 'tonic v
tonic-build v
prost v
prost-build v
axum v0.7
tower v0.4'
```
