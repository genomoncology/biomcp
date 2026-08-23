# Make contradictory variant-filter validation reliable

The full routine `make test` run timed out after 10 seconds in
`tests/json_error_contract.rs::contradictory_variant_filters_fail_before_myvariant_contact`.
The test must continue rejecting contradictory `--has`/`--missing` filters before
contacting MyVariant; determine whether startup or parallel-suite contention is
responsible and make this guarantee reliable without weakening its assertions.
