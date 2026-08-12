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
cat "${BIOMCP_SPEC_CARGO_TREE:?spec preparation did not export dependency evidence}" | mustmatch not like 'tonic v
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

This is a claim about the targets' own defaults, so the probe clears every
build variable an outer gate may have overridden. The release gate keeps this
routine graph intact, then adds a separate named full-feature proof.

```bash
env -u BIOMCP_BIN -u SPEC_BIN -u MAKEFLAGS -u MAKEOVERRIDES \
    -u ROUTINE_CARGO_FEATURES -u SPEC_PROFILE -u SPEC_USE_PROVIDED_BIN \
  make -C ../.. -n test | mustmatch like 'cargo nextest archive --locked --no-default-features
cargo nextest run --archive-file'
env -u BIOMCP_BIN -u SPEC_BIN -u MAKEFLAGS -u MAKEOVERRIDES \
    -u ROUTINE_CARGO_FEATURES -u SPEC_PROFILE -u SPEC_USE_PROVIDED_BIN \
  make -C ../.. -n spec | mustmatch like 'scripts/run-specs.sh spec'
rg --fixed-strings 'ROUTINE_CARGO_FEATURES' ../../Makefile ../../bin/lint \
  ../../scripts/run-specs.sh | mustmatch like 'ROUTINE_CARGO_FEATURES'
```

The matching claim about what each binary *says* — that a feature-off build
reports the prediction as not built, and a feature-on build names the key —
is a property of the binary under test, not of the routine profile. It cannot
live on this page, because `release-gate` runs these same pages against the
release binary. It is proven natively for both builds by
`list_variant_explains_alphagenome_availability_for_this_build`.

## Release artifacts retain the AlphaGenome feature

The default package feature keeps the shipped client's runtime dependencies
together. Protobuf generation is maintainer-only and must not return to the
normal AlphaGenome feature graph. This metadata check distinguishes a real
optional runtime feature from unconditional or generator-only dependencies.

```bash
cat "${BIOMCP_SPEC_CARGO_METADATA:?spec preparation did not export package metadata}" | jq '[.packages[] | select(.name == "biomcp-cli") | .features][0] | ((.default | index("alphagenome")) != null and (.alphagenome | index("dep:tonic")) != null and (.alphagenome | index("dep:prost")) != null and (.alphagenome | index("dep:zstd")) != null and (.alphagenome | index("dep:tonic-build")) == null)' | mustmatch 'true'
```
