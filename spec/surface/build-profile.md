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

## Routine gates select the smaller feature set

The ordinary test and spec targets must select this graph themselves. Calling
Cargo with a smaller feature set by hand is not enough if either routine gate
silently restores the default client.

```bash
make -C ../.. -n test | mustmatch like 'cargo nextest run --no-default-features'
make -C ../.. -n spec | mustmatch like 'cargo build --locked --no-default-features --profile'
```

## Release artifacts retain the AlphaGenome feature

The default package feature keeps the shipped client and all of its runtime and
build-time dependencies together. This metadata check distinguishes a real
optional feature from an unconditional dependency that only happens to compile.

```bash
cargo metadata --locked --no-deps --format-version 1 | jq '[.packages[] | select(.name == "biomcp-cli") | .features][0] | ((.default | index("alphagenome")) != null and (.alphagenome | index("dep:tonic")) != null and (.alphagenome | index("dep:prost")) != null and (.alphagenome | index("dep:zstd")) != null and (.alphagenome | index("dep:tonic-build")) != null)' | mustmatch 'true'
```
