# MkDocs 2.0 warns about Material compatibility

`make test` and `make spec` emit the existing strict-documentation warning that
MkDocs 2.0 is incompatible with Material for MkDocs. This is in the MkDocs
build configuration and is outside the catalog-measurement attribution change.
Track the migration or supported-version pin separately so the warning does not
become a documentation-build failure.
